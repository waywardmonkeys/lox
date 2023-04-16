//! Abstractions over numerical types and vector-like types.


use std::{
    fmt::Debug,
};

use cgmath::BaseFloat;
use num_traits::{Float, FloatConst, Num, NumAssign, NumCast};

use crate::{
    cast::{self, CastFrom, CastInto, Fidelity},
};


/// Primitive numerical types, like `f64` and `u32`.
///
/// This trait is automatically implemented for all types that satisfy the
/// super-trait constraints.
///
/// Note that this is very similar to `cgmath::BaseNum`. Right now, the only
/// difference is the additional `'static` bound and the `PrimitiveCast` bound.
pub trait PrimitiveNum:
    'static + Copy + Debug + Num + PartialOrd + NumAssign + NumCast + PrimitiveCast<cast::Lossy>
{}

impl<T> PrimitiveNum for T
where
    T: 'static + Copy + Debug + Num + PartialOrd + NumAssign + NumCast + PrimitiveCast<cast::Lossy>,
{}

/// Primitive floating point types: `f32` and `f64`.
///
/// This trait is automatically implemented for all types that satisfy the
/// super-trait constraints.
pub trait PrimitiveFloat:
    PrimitiveNum
        + Float
        + FloatConst
        + BaseFloat
        + CastFrom<f32, Fidelity = cast::Lossless>
        + CastFrom<u8, Fidelity = cast::Lossless>
        + CastFrom<i8, Fidelity = cast::Lossless>
        + CastFrom<u16, Fidelity = cast::Lossless>
        + CastFrom<i16, Fidelity = cast::Lossless>
{
    /// Creates `Self` from the given `f32`. Uses `cast::lossless` internally.
    fn from_f32(v: f32) -> Self {
        cast::lossless(v)
    }
}

impl<T> PrimitiveFloat for T
where
    T: PrimitiveNum
        + Float
        + FloatConst
        + BaseFloat
        + CastFrom<f32, Fidelity = cast::Lossless>
        + CastFrom<u8, Fidelity = cast::Lossless>
        + CastFrom<i8, Fidelity = cast::Lossless>
        + CastFrom<u16, Fidelity = cast::Lossless>
        + CastFrom<i16, Fidelity = cast::Lossless>
{}


/// Types that can be casted from and into all primitive types.
///
/// This is basically a trait-bound alias that is automatically implemented for
/// all types that satisfy the supertrait bounds.
pub trait PrimitiveCast<F: Fidelity>: CastFromPrimitive<F> + CastIntoPrimitive<F> {}

impl<T, F: Fidelity> PrimitiveCast<F> for T where T: CastFromPrimitive<F> + CastIntoPrimitive<F> {}

pub trait CastFromPrimitive<F: Fidelity>:
    CastFrom<u8, Fidelity: cast::SufficientFor<F>>
    + CastFrom<i8, Fidelity: cast::SufficientFor<F>>
    + CastFrom<u16, Fidelity: cast::SufficientFor<F>>
    + CastFrom<i16, Fidelity: cast::SufficientFor<F>>
    + CastFrom<u32, Fidelity: cast::SufficientFor<F>>
    + CastFrom<i32, Fidelity: cast::SufficientFor<F>>
    + CastFrom<u64, Fidelity: cast::SufficientFor<F>>
    + CastFrom<i64, Fidelity: cast::SufficientFor<F>>
    + CastFrom<u128, Fidelity: cast::SufficientFor<F>>
    + CastFrom<i128, Fidelity: cast::SufficientFor<F>>
    + CastFrom<f32, Fidelity: cast::SufficientFor<F>>
    + CastFrom<f64, Fidelity: cast::SufficientFor<F>>
{}

impl<T, F: Fidelity> CastFromPrimitive<F> for T
where
    T: CastFrom<u8, Fidelity: cast::SufficientFor<F>>
        + CastFrom<i8, Fidelity: cast::SufficientFor<F>>
        + CastFrom<u16, Fidelity: cast::SufficientFor<F>>
        + CastFrom<i16, Fidelity: cast::SufficientFor<F>>
        + CastFrom<u32, Fidelity: cast::SufficientFor<F>>
        + CastFrom<i32, Fidelity: cast::SufficientFor<F>>
        + CastFrom<u64, Fidelity: cast::SufficientFor<F>>
        + CastFrom<i64, Fidelity: cast::SufficientFor<F>>
        + CastFrom<u128, Fidelity: cast::SufficientFor<F>>
        + CastFrom<i128, Fidelity: cast::SufficientFor<F>>
        + CastFrom<f32, Fidelity: cast::SufficientFor<F>>
        + CastFrom<f64, Fidelity: cast::SufficientFor<F>>
{}

pub trait CastIntoPrimitive<F: Fidelity>:
    CastInto<u8, Fidelity: cast::SufficientFor<F>>
    + CastInto<i8, Fidelity: cast::SufficientFor<F>>
    + CastInto<u16, Fidelity: cast::SufficientFor<F>>
    + CastInto<i16, Fidelity: cast::SufficientFor<F>>
    + CastInto<u32, Fidelity: cast::SufficientFor<F>>
    + CastInto<i32, Fidelity: cast::SufficientFor<F>>
    + CastInto<u64, Fidelity: cast::SufficientFor<F>>
    + CastInto<i64, Fidelity: cast::SufficientFor<F>>
    + CastInto<u128, Fidelity: cast::SufficientFor<F>>
    + CastInto<i128, Fidelity: cast::SufficientFor<F>>
    + CastInto<f32, Fidelity: cast::SufficientFor<F>>
    + CastInto<f64, Fidelity: cast::SufficientFor<F>>
{}

impl<T, F: Fidelity> CastIntoPrimitive<F> for T
where
    T: CastInto<u8, Fidelity: cast::SufficientFor<F>>
        + CastInto<i8, Fidelity: cast::SufficientFor<F>>
        + CastInto<u16, Fidelity: cast::SufficientFor<F>>
        + CastInto<i16, Fidelity: cast::SufficientFor<F>>
        + CastInto<u32, Fidelity: cast::SufficientFor<F>>
        + CastInto<i32, Fidelity: cast::SufficientFor<F>>
        + CastInto<u64, Fidelity: cast::SufficientFor<F>>
        + CastInto<i64, Fidelity: cast::SufficientFor<F>>
        + CastInto<u128, Fidelity: cast::SufficientFor<F>>
        + CastInto<i128, Fidelity: cast::SufficientFor<F>>
        + CastInto<f32, Fidelity: cast::SufficientFor<F>>
        + CastInto<f64, Fidelity: cast::SufficientFor<F>>
{}
