
use crate::util::Recip;

use num::One;

use std::ops::{Add, Mul, Neg};

/// Trait defining elements that are arithmetic enough to be used in
/// matrix operations.
///
/// A [`Matrix`](super::Matrix) is NOT required to contain elements
/// satisfying this trait in general, but certain operations do
/// require it.
///
/// Structs needn't implement this trait by hand, as a blanket impl
/// takes care of it for any satisfactory type.
pub trait MatrixElement: Clone + One + for<'a> Add<&'a Self, Output=Self> + for<'a> Mul<&'a Self, Output=Self> + Neg<Output=Self> {}

/// Trait for elements that are both [`MatrixElement`] and [`Recip`].
/// This is the trait required to be able to fully row reduce a
/// matrix. As with [`MatrixElement`], a blanket impl takes any
/// eligible types to this trait automatically, so manual
/// implementations are not necessary.
pub trait MatrixFieldElement: MatrixElement + Recip<Output=Self> {}

impl<T> MatrixElement for T where T: Clone + One + for<'a> Add<&'a Self, Output=Self> + for<'a> Mul<&'a Self, Output=Self> + Neg<Output=Self> {}

impl<T> MatrixFieldElement for T where T: MatrixElement + Recip<Output=T> {}
