mod layers;
mod training;

use dfdx::{data::IteratorBatchExt, optim::Adam, prelude::*};
pub use layers::*;
pub use training::*;

use std::ops::Deref;

pub type E = f32;
pub type D = Cuda;

const CHANNELS: usize = 4;
pub(crate) const BATCH_SIZE: usize = 256;

type Conv<const I: usize, const O: usize> = (Conv2D<I, O, 3, 1, 1>, Bias2D<O>, Selu);

type UNetBlock<const C1: usize, const C2: usize, M> =
    Upscale2DResidual<((Conv2D<C1, C2, 3, 2, 1>, Bias2D<C2>, Selu), M, Conv<C2, C1>)>;

// Specialized UNet that uses 1x1 convolutions to emulate a pixel-wise MLP.
type UNetInternalBlock<const C1: usize, const C2: usize> = Upscale2DResidual<(
    (Conv2D<C1, C2, 3, 2, 1>, Bias2D<C2>, Selu),
    (Conv2D<C2, C2, 1, 1, 0>, Bias2D<C2>, Selu),
    (Conv2D<C2, C1, 1, 1, 0>, Bias2D<C1>, Selu),
)>;

type TinyNet = (
    SplitInto<(
        Conv2D<CHANNELS, 1, 3, 1, 1>,
        (Conv2D<CHANNELS, 1, 3, 1, 1>, AvgPoolGlobal, Sigmoid),
    )>,
);

type BeginnerNet = (
    Conv<CHANNELS, 16>,
    Residual<(
        Residual<Repeated<Conv<16, 16>, 4>>,
        UNetBlock<16, 32, UNetBlock<32, 64, UNetInternalBlock<64, 128>>>,
        Residual<Repeated<Conv<16, 16>, 4>>,
    )>,
    SplitInto<(
        Conv2D<16, 1, 3, 1, 1>,
        (Conv2D<16, 1, 3, 1, 1>, AvgPoolGlobal, Sigmoid),
    )>,
);

type BeginnerNet2 = (
    Conv<CHANNELS, 32>,
    Residual<(
        Residual<Repeated<Conv<32, 32>, 4>>,
        UNetBlock<32, 64, UNetBlock<64, 128, UNetInternalBlock<128, 256>>>,
        Residual<Repeated<Conv<32, 32>, 4>>,
    )>,
    SplitInto<(
        Conv2D<32, 1, 3, 1, 1>,
        (Conv2D<32, 1, 3, 1, 1>, AvgPoolGlobal, Sigmoid),
    )>,
);

type BeginnerNet3 = (
    Conv<CHANNELS, 64>,
    Residual<(
        Residual<Repeated<Conv<64, 64>, 4>>,
        UNetBlock<64, 128, UNetBlock<128, 256, UNetInternalBlock<256, 512>>>,
        Residual<Repeated<Conv<64, 64>, 4>>,
    )>,
    SplitInto<(
        Conv2D<64, 1, 3, 1, 1>,
        (Conv2D<64, 1, 3, 1, 1>, AvgPoolGlobal, Sigmoid),
    )>,
);

type ExpertNet = (
    Conv<CHANNELS, 16>,
    Residual<(
        Residual<Repeated<Conv<16, 16>, 4>>,
        UNetBlock<16, 32, UNetBlock<32, 64, UNetBlock<64, 128, UNetInternalBlock<128, 256>>>>,
        Residual<Repeated<Conv<16, 16>, 4>>,
    )>,
    SplitInto<(
        Conv2D<16, 1, 3, 1, 1>,
        (Conv2D<16, 1, 3, 1, 1>, AvgPoolGlobal, Sigmoid),
    )>,
);

pub const WIDTH: usize = 9;
pub const HEIGHT: usize = 9;

pub type Net = BeginnerNet3;
pub type BuiltNet = <Net as BuildOnDevice<D, E>>::Built;

pub struct NNEvalFunction {
    pub net: BuiltNet,
    pub dev: D,
    pub optimizer: Adam<BuiltNet, E, D>,
}

pub fn batch_to_tensor<const C: usize>(
    batch: &[impl Deref<Target = [E]>],
    dev: &D,
) -> Tensor<(usize, Const<C>, Const<9>, Const<9>), E, D> {
    for b in batch {
        assert_eq!(b.len(), C * WIDTH * HEIGHT);
    }

    let flattened = batch
        .iter()
        .flat_map(|x| x.iter())
        .copied()
        .collect::<Vec<_>>();

    let shape = (batch.len(), Const, Const, Const);

    dev.tensor_from_vec(flattened, shape)
}

pub fn tensor_to_batch(tensor: Tensor<impl Shape, E, D>) -> Vec<Vec<E>> {
    tensor
        .as_vec()
        .into_iter()
        .batch_exact(WIDTH * HEIGHT)
        .collect()
}

// use crate::search::EvalFunction;

// impl EvalFunction for NNEvalFunction {
// fn eval_batch(
// &self,
// features: &[impl Deref<Target = [f64]>],
// masks: &[impl Deref<Target = [f64]>],
// ) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
// let features = batch_to_tensor::<CHANNELS>(features, &self.dev);
// let masks = batch_to_tensor::<1>(masks, &self.dev);

// let (mut values, mut policies) = self.net.forward(features);
// values = values * masks.clone();
// policies = (policies - (masks.recip() - 1.)).softmax::<Axes2<2, 3>>();

// (tensor_to_batch(values), tensor_to_batch(policies))
// }

// fn train_batch(
// &mut self,
// features: &[impl Deref<Target = [f64]>],
// masks: &[impl Deref<Target = [f64]>],
// values: &[impl Deref<Target = [f64]>],
// policies: &[impl Deref<Target = [f64]>],
// ) {
// let features = batch_to_tensor::<CHANNELS>(features, &self.dev);
// let masks = batch_to_tensor::<1>(masks, &self.dev);
// let expected_values = batch_to_tensor::<1>(values, &self.dev);
// let expected_policies = batch_to_tensor::<1>(policies, &self.dev);

// let (mut values, mut policies) = self.net.forward_mut(features.leaky_trace());
// values = values * masks.clone();
// policies = (policies - (masks.recip() - 1.)).softmax::<Axes2<2, 3>>();

// let policies_0 = policies
// .retaped::<NoneTape>()
// .select(self.dev.tensor(0))
// .to_dtype::<f64>()
// .as_vec();
// let expected_policies_0 = expected_policies
// .clone()
// .select(self.dev.tensor(0))
// .to_dtype::<f64>()
// .as_vec();

// crate::print_probs_2d(&policies_0, WIDTH);
// println!();
// crate::print_probs_2d(&expected_policies_0, WIDTH);

// let value_err = (values - expected_values).square().sum::<(), _>();
// let policy_err = (policies - expected_policies).square().sum::<(), _>();

// println!(
// "value_err: {}, policy_err: {}",
// value_err.array(),
// policy_err.array()
// );
// let err = value_err + policy_err * 0.2;
// let grads = err.backward();
// self.optimizer.update(&mut self.net, &grads).unwrap();
// }
// }

// pub struct DummyEvalFunction;

// impl EvalFunction for DummyEvalFunction {
// fn eval_batch(
// &self,
// _features: &[impl Deref<Target = [f64]>],
// masks: &[impl Deref<Target = [f64]>],
// ) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
// let values = masks.iter().map(|s| s.to_vec()).collect();
// let policies = masks
// .iter()
// .map(|mask| {
// let total = mask.iter().sum::<f64>();
// mask.iter().map(|&m| m / total).collect()
// })
// .collect();

// (values, policies)
// }

// fn train_batch(
// &mut self,
// _features: &[impl Deref<Target = [f64]>],
// _masks: &[impl Deref<Target = [f64]>],
// _values: &[impl Deref<Target = [f64]>],
// _policies: &[impl Deref<Target = [f64]>],
// ) {
// }
// }

pub fn test() {
    let dev = Cuda::default();
    let nn = dev.build_module::<Net, f32>();
    println!("{}", nn.num_trainable_params());
}
