use dfdx::prelude::*;

#[derive(Clone)]
pub struct Upscale2DResidual<M>(M);

impl<E: Dtype, D: Device<E>, M: TensorCollection<E, D>> TensorCollection<E, D>
    for Upscale2DResidual<M>
{
    type To<E2: Dtype, D2: Device<E2>> = Upscale2DResidual<M::To<E2, D2>>;

    fn iter_tensors<V: ModuleVisitor<Self, E, D>>(
        visitor: &mut V,
    ) -> Result<Option<Self::To<V::E2, V::D2>>, V::Err> {
        visitor.visit_fields(Self::module("0", |s| &s.0, |s| &mut s.0), Upscale2DResidual)
    }
}

impl<E: Dtype, D: Device<E>, M: BuildOnDevice<D, E>> BuildOnDevice<D, E> for Upscale2DResidual<M> {
    type Built = Upscale2DResidual<M::Built>;
}

impl<I: WithEmptyTape + TryAdd + HasShape, M: Module<I, Error = I::Err>> Module<I>
    for Upscale2DResidual<M>
where
    M::Output: GenericUpscale2D<Bilinear> + TryUpscale2D + HasErr<Err = I::Err>,
    <M::Output as GenericUpscale2D<Bilinear>>::Output<usize, usize>:
        HasShape<WithShape<I::Shape> = I> + RealizeTo + std::fmt::Debug,
    <<M::Output as GenericUpscale2D<Bilinear>>::Output<usize, usize> as HasShape>::Shape:
        Shape<Concrete = <I::Shape as Shape>::Concrete>,
{
    type Output = I;
    type Error = M::Error;

    fn try_forward(&self, input: I) -> Result<I, Self::Error> {
        let residual = input.with_empty_tape();
        let shape = input.shape().concrete();
        let height = shape[I::Shape::NUM_DIMS - 2];
        let width = shape[I::Shape::NUM_DIMS - 1];
        let output = self
            .0
            .try_forward(input)?
            .try_upscale2d_like(Bilinear, height, width)?
            .realize::<I::Shape>()
            .unwrap();
        output.try_add(residual)
    }
}

impl<I: WithEmptyTape + TryAdd + HasShape, M: ModuleMut<I, Error = I::Err>> ModuleMut<I>
    for Upscale2DResidual<M>
where
    M::Output: GenericUpscale2D<Bilinear> + TryUpscale2D + HasErr<Err = I::Err>,
    <M::Output as GenericUpscale2D<Bilinear>>::Output<usize, usize>:
        HasShape<WithShape<I::Shape> = I> + RealizeTo + std::fmt::Debug,
    <<M::Output as GenericUpscale2D<Bilinear>>::Output<usize, usize> as HasShape>::Shape:
        Shape<Concrete = <I::Shape as Shape>::Concrete>,
{
    type Output = I;
    type Error = M::Error;

    fn try_forward_mut(&mut self, input: I) -> Result<I, Self::Error> {
        let residual = input.with_empty_tape();
        let shape = input.shape().concrete();
        let height = shape[I::Shape::NUM_DIMS - 2];
        let width = shape[I::Shape::NUM_DIMS - 1];
        let output = self
            .0
            .try_forward_mut(input)?
            .try_upscale2d_like(Bilinear, height, width)?
            .realize::<I::Shape>()
            .unwrap();
        output.try_add(residual)
    }
}

#[derive(Clone, Copy, Default)]
pub struct Selu {}

impl ZeroSizedModule for Selu {}
impl NonMutableModule for Selu {}

impl<S: Shape, E: Dtype, D: Device<E>, T: Tape<E, D>> Module<Tensor<S, E, D, T>> for Selu {
    type Output = Tensor<S, E, D, T>;
    type Error = D::Err;

    fn try_forward(&self, input: Tensor<S, E, D, T>) -> Result<Tensor<S, E, D, T>, Self::Error> {
        let zero = E::from_f32(0.0).unwrap();
        let one = E::from_f32(1.0).unwrap();
        let neg_inf = E::from_f32(f32::NEG_INFINITY).unwrap();

        let alpha = E::from_f64(1.6732632423543772).unwrap();
        let scale = E::from_f64(1.0507009873554804).unwrap();

        (input.with_empty_tape().try_relu()?
            + input
                .try_exp()?
                .try_sub(one)?
                .try_mul(alpha)?
                .try_clamp(neg_inf, zero)?)
        .try_mul(scale)
    }
}
