mod layers;

use dfdx::{data::IteratorBatchExt, optim::Adam, prelude::*};
pub use layers::*;

use std::ops::Deref;

type E = f32;
type D = Cuda;

const CHANNELS: usize = 4;

type Conv<const I: usize, const O: usize> = (Conv2D<I, O, 3, 1, 1>, Bias2D<O>, Selu);

type UNetBlock<const C1: usize, const C2: usize, M> =
    Upscale2DResidual<((Conv2D<C1, C2, 3, 2, 1>, Bias2D<C2>, Selu), M, Conv<C2, C1>)>;

type BeginnerNet = (
    Conv<CHANNELS, 16>,
    Residual<(
        Residual<Repeated<Conv<16, 16>, 4>>,
        UNetBlock<16, 32, UNetBlock<32, 64, UNetBlock<64, 128, Conv<128, 128>>>>,
        Residual<Repeated<Conv<16, 16>, 4>>,
    )>,
    SplitInto<(Conv2D<16, 1, 3, 1, 1>, Conv2D<16, 1, 3, 1, 1>)>,
);

type ExpertNet = (
    Conv<CHANNELS, 16>,
    Residual<(
        Residual<Repeated<Conv<16, 16>, 4>>,
        UNetBlock<
            16,
            32,
            UNetBlock<32, 64, UNetBlock<64, 128, UNetBlock<128, 256, Conv<256, 256>>>>,
        >,
        Residual<Repeated<Conv<16, 16>, 4>>,
    )>,
    SplitInto<(Conv2D<16, 1, 3, 1, 1>, Conv2D<16, 1, 3, 1, 1>)>,
);

const WIDTH: usize = 10;
const HEIGHT: usize = 10;

pub type Net = BeginnerNet;
pub type BuiltNet = <Net as BuildOnDevice<D, E>>::Built;

pub struct NNEvalFunction {
    pub net: BuiltNet,
    pub dev: D,
    pub optimizer: Adam<BuiltNet, E, D>,
}

fn batch_to_tensor<const C: usize>(
    batch: &[impl Deref<Target = [f64]>],
    dev: &D,
) -> Tensor<(usize, Const<C>, Const<HEIGHT>, Const<WIDTH>), E, D> {
    assert!(batch.iter().all(|channel| channel.len() == WIDTH * HEIGHT));

    let flattened = batch
        .iter()
        .flat_map(|x| x.iter())
        .map(|x| *x as E)
        .collect::<Vec<_>>();

    let shape = (batch.len(), Const, Const, Const);

    dev.tensor_from_vec(flattened, shape)
}

fn tensor_to_batch<S: Shape>(tensor: Tensor<S, E, D>) -> Vec<Vec<f64>> {
    tensor
        .as_vec()
        .into_iter()
        .map(|x| x as f64)
        .batch_exact(WIDTH * HEIGHT)
        .collect()
}

use crate::search::EvalFunction;

impl EvalFunction for NNEvalFunction {
    fn eval_batch(
        &self,
        features: &[impl Deref<Target = [f64]>],
        masks: &[impl Deref<Target = [f64]>],
    ) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
        let features = batch_to_tensor::<CHANNELS>(features, &self.dev);
        let masks = batch_to_tensor::<1>(masks, &self.dev);

        let (mut values, mut policies) = self.net.forward(features);
        values = values * masks.clone();
        policies = (policies - (masks.recip() - 1.)).softmax::<Axes2<2, 3>>();

        (tensor_to_batch(values), tensor_to_batch(policies))
    }

    fn train_batch(
        &mut self,
        features: &[impl Deref<Target = [f64]>],
        masks: &[impl Deref<Target = [f64]>],
        values: &[impl Deref<Target = [f64]>],
        policies: &[impl Deref<Target = [f64]>],
    ) {
        let features = batch_to_tensor::<CHANNELS>(features, &self.dev);
        let masks = batch_to_tensor::<1>(masks, &self.dev);
        let expected_values = batch_to_tensor::<1>(values, &self.dev);
        let expected_policies = batch_to_tensor::<1>(policies, &self.dev);

        let (mut values, mut policies) = self.net.forward_mut(features.leaky_traced());
        values = values * masks.clone();
        policies = (policies - (masks.recip() - 1.)).softmax::<Axes2<2, 3>>();

        let value_err = (values - expected_values).square().sum::<(), _>();
        let policy_err = (policies - expected_policies).square().sum::<(), _>();

        let err = value_err + policy_err;
        let grads = err.backward();
        self.optimizer.update(&mut self.net, &grads).unwrap();
    }
}

pub struct DummyEvalFunction;

impl EvalFunction for DummyEvalFunction {
    fn eval_batch(
        &self,
        _features: &[impl Deref<Target = [f64]>],
        masks: &[impl Deref<Target = [f64]>],
    ) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
        let values = masks.iter().map(|s| s.to_vec()).collect();
        let policies = masks
            .iter()
            .map(|mask| {
                let total = mask.iter().sum::<f64>();
                mask.iter().map(|&m| m / total).collect()
            })
            .collect();

        (values, policies)
    }

    fn train_batch(
        &mut self,
        _features: &[impl Deref<Target = [f64]>],
        _masks: &[impl Deref<Target = [f64]>],
        _values: &[impl Deref<Target = [f64]>],
        _policies: &[impl Deref<Target = [f64]>],
    ) {
    }
}
