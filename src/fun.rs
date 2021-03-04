use core::pin::Pin;
use core::task::Context;
use core::task::Poll;
use core::cmp::Ordering;
use core::future::{Future, Ready, ready};

use gc::{Gc, GcCell, Trace, Finalize, custom_trace};
use gc_derive::{Trace, Finalize};

use guvm_rs::{Value, BuiltInAsyncFunction, BuiltInSynchronousFunction, Closure, Arity, VirtualMachine};

use crate::{V, ValueBaseOrdered, ValueBase, VvvmFailure, VvvmFuture};
mod util;
mod value;
mod order;
mod boolean;
mod float;
mod int;

#[derive(Finalize)]
pub enum Fun<SS, SA, DS, DA, F, Fut>
where
    SS: ValueBaseOrdered,
    DS: ValueBase,
    SA: ValueBaseOrdered,
    DA: ValueBase,
    F: 'static,
    Fut: 'static,
{
    SynchronousFunction(SynchronousFun<SS, DS>),
    AsynchronousFunction(AsynchronousFun<SA, DA>),
    Closure(Closure<V<SS, SA, DS, DA, F, Fut>>),
}

unsafe impl<SS, SA, DS, DA, F, Fut> Trace for Fun<SS, SA, DS, DA, F, Fut>
where
    SS: ValueBaseOrdered,
    DS: ValueBase,
    SA: ValueBaseOrdered,
    DA: ValueBase,
    F: 'static,
    Fut: 'static,
{
    custom_trace!(this, {
        match this {
            Fun::SynchronousFunction(f) => mark(f),
            Fun::AsynchronousFunction(f) => mark(f),
            Fun::Closure(f) => mark(f),
        }
    });
}

impl<SS, SA, DS, DA, F, Fut> Clone for Fun<SS, SA, DS, DA, F, Fut>
where
    SS: ValueBaseOrdered + Clone,
    DS: ValueBase + Clone,
    SA: ValueBaseOrdered + Clone,
    DA: ValueBase + Clone,
{
    fn clone(&self) -> Self {
        match self {
            Fun::SynchronousFunction(f) => Fun::SynchronousFunction(f.clone()),
            Fun::AsynchronousFunction(f) => Fun::AsynchronousFunction(f.clone()),
            Fun::Closure(f) => Fun::Closure(f.clone()),
        }
    }
}

impl<SS, SA, DS, DA, F, Fut> PartialEq for Fun<SS, SA, DS, DA, F, Fut>
where
    SS: ValueBaseOrdered,
    DS: ValueBase,
    SA: ValueBaseOrdered,
    DA: ValueBase,
    F: 'static,
    Fut: 'static,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Fun::SynchronousFunction(a), Fun::SynchronousFunction(b)) => a == b,
            (Fun::AsynchronousFunction(a), Fun::AsynchronousFunction(b)) => a == b,
            (Fun::Closure(a), Fun::Closure(b)) => a.ordinal() == b.ordinal(),
            _ => false,
        }
    }
}

impl<SS, SA, DS, DA, F, Fut> Eq for Fun<SS, SA, DS, DA, F, Fut>
where
    SS: ValueBaseOrdered,
    DS: ValueBase,
    SA: ValueBaseOrdered,
    DA: ValueBase,
    F: 'static,
    Fut: 'static,
{}

impl<SS, SA, DS, DA, F, Fut> PartialOrd for Fun<SS, SA, DS, DA, F, Fut>
where
    SS: ValueBaseOrdered,
    DS: ValueBase,
    SA: ValueBaseOrdered,
    DA: ValueBase,
    F: 'static,
    Fut: 'static,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<SS, SA, DS, DA, F, Fut> Ord for Fun<SS, SA, DS, DA, F, Fut>
where
    SS: ValueBaseOrdered,
    DS: ValueBase,
    SA: ValueBaseOrdered,
    DA: ValueBase,
    F: 'static,
    Fut: 'static,
{
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Fun::SynchronousFunction(a), Fun::SynchronousFunction(b)) => a.cmp(b),
            (Fun::AsynchronousFunction(a), Fun::AsynchronousFunction(b)) => a.cmp(b),

            (Fun::Closure(a), Fun::Closure(b)) => match (a.is_asynchronous(), b.is_asynchronous()) {
                (false, true) => Ordering::Less,
                (true, false) => Ordering::Greater,
                _ => a.ordinal().cmp(&b.ordinal()),
            }

            (Fun::Closure(a), Fun::SynchronousFunction(SynchronousFun::Dynamic {ordinal: b, ..})) => {
                if a.is_asynchronous() {
                    Ordering::Greater
                } else {
                    a.ordinal().cmp(b)
                }
            }
            (Fun::SynchronousFunction(SynchronousFun::Dynamic {ordinal: a, ..}), Fun::Closure(b)) => {
                if b.is_asynchronous() {
                    Ordering::Less
                } else {
                    a.cmp(&b.ordinal())
                }
            }

            (Fun::Closure(a), Fun::AsynchronousFunction(AsynchronousFun::Dynamic {ordinal: b, ..})) => {
                if a.is_asynchronous() {
                    a.ordinal().cmp(b)
                } else {
                    Ordering::Less
                }
            }
            (Fun::AsynchronousFunction(AsynchronousFun::Dynamic {ordinal: a, ..}), Fun::Closure(b)) => {
                if b.is_asynchronous() {
                    a.cmp(&b.ordinal())
                } else {
                    Ordering::Greater
                }
            }

            (Fun::SynchronousFunction(_), _) => Ordering::Less,
            (Fun::Closure(a), _) if !a.is_asynchronous() => Ordering::Less,

            (Fun::AsynchronousFunction(_), _) => Ordering::Less,
            (Fun::Closure(_), _) => Ordering::Greater,
        }
    }
}

#[derive(Clone, Trace, Finalize)]
pub enum SynchronousFun<S, D> {
    Core(SynchronousCoreFunction),
    StaticSynchronous(S),
    Dynamic {
        ordinal: usize,
        fun: DynamicSynchronous<D>,
    },
}

impl<S: PartialEq, D> PartialEq for SynchronousFun<S, D> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SynchronousFun::Core(a), SynchronousFun::Core(b)) => a == b,
            (SynchronousFun::StaticSynchronous(a), SynchronousFun::StaticSynchronous(b)) => a == b,
            (SynchronousFun::Dynamic {ordinal: a, ..}, SynchronousFun::Dynamic {ordinal: b, ..}) => a == b,
            _ => false,
        }
    }
}

impl<S: Eq, D> Eq for SynchronousFun<S, D> {}

impl<S: Ord, D> PartialOrd for SynchronousFun<S, D> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<S: Ord, D> Ord for SynchronousFun<S, D> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (SynchronousFun::Core(a), SynchronousFun::Core(b)) => a.cmp(b),
            (SynchronousFun::Core(_), _) => Ordering::Less,
            (SynchronousFun::StaticSynchronous(a), SynchronousFun::StaticSynchronous(b)) => a.cmp(b),
            (SynchronousFun::StaticSynchronous(_), _) => Ordering::Less,
            (SynchronousFun::Dynamic {ordinal: a, ..}, SynchronousFun::Dynamic {ordinal: b, ..}) => a.cmp(b),
            (SynchronousFun::Dynamic {..}, _) => Ordering::Greater,
        }
    }
}

impl<SS, SA, DS, DA, F, Fut> BuiltInSynchronousFunction<V<SS, SA, DS, DA, F, Fut>, VvvmFailure<V<SS, SA, DS, DA, F, Fut>, F>> for SynchronousFun<SS, DS> where
    SS: ValueBaseOrdered + BuiltInSynchronousFunction<V<SS, SA, DS, DA, F, Fut>, VvvmFailure<V<SS, SA, DS, DA, F, Fut>, F>>,
    DS: ValueBase + BuiltInSynchronousFunction<V<SS, SA, DS, DA, F, Fut>, VvvmFailure<V<SS, SA, DS, DA, F, Fut>, F>>,
    SA: ValueBaseOrdered,
    DA: ValueBase,
    F: 'static,
    Fut: 'static,
{
    fn arity(&self) -> Arity {
        match self {
            SynchronousFun::Core(f) => BuiltInSynchronousFunction::<V<SS, SA, DS, DA, F, Fut>, CoreFailure<V<SS, SA, DS, DA, F, Fut>>>::arity(f),
            SynchronousFun::StaticSynchronous(ss) => ss.arity(),
            SynchronousFun::Dynamic {fun: ds, ..} => ds.arity(),
        }
    }

    fn invoke(
        &mut self,
        args: &[V<SS, SA, DS, DA, F, Fut>],
        vm: &mut VirtualMachine<V<SS, SA, DS, DA, F, Fut>>,
    ) -> Result<V<SS, SA, DS, DA, F, Fut>, VvvmFailure<V<SS, SA, DS, DA, F, Fut>, F>> {
        match self {
            SynchronousFun::Core(f) => f.invoke(args, vm).map_err(|e| VvvmFailure::Core(e)),
            SynchronousFun::StaticSynchronous(ss) => ss.invoke(args, vm),
            SynchronousFun::Dynamic {fun: ds, ..} => ds.invoke(args, vm),
        }
    }
}

#[derive(Clone, Trace, Finalize)]
pub enum DynamicSynchronous<D> {
    Core(DynamicCoreFunction),
    Custom(D),
}

impl<SS, SA, DS, DA, F, Fut> BuiltInSynchronousFunction<V<SS, SA, DS, DA, F, Fut>, VvvmFailure<V<SS, SA, DS, DA, F, Fut>, F>> for DynamicSynchronous<DS> where
    SS: ValueBaseOrdered,
    DS: ValueBase + BuiltInSynchronousFunction<V<SS, SA, DS, DA, F, Fut>, VvvmFailure<V<SS, SA, DS, DA, F, Fut>, F>>,
    SA: ValueBaseOrdered,
    DA: ValueBase,
    F: 'static,
    Fut: 'static,
{
    fn arity(&self) -> Arity {
        match self {
            DynamicSynchronous::Core(f) => BuiltInSynchronousFunction::<V<SS, SA, DS, DA, F, Fut>, CoreFailure<V<SS, SA, DS, DA, F, Fut>>>::arity(f),
            DynamicSynchronous::Custom(ds) => ds.arity(),
        }
    }

    fn invoke(
        &mut self,
        args: &[V<SS, SA, DS, DA, F, Fut>],
        vm: &mut VirtualMachine<V<SS, SA, DS, DA, F, Fut>>,
    ) -> Result<V<SS, SA, DS, DA, F, Fut>, VvvmFailure<V<SS, SA, DS, DA, F, Fut>, F>> {
        match self {
            DynamicSynchronous::Core(f) => f.invoke(args, vm).map_err(|e| VvvmFailure::Core(e)),
            DynamicSynchronous::Custom(ds) => ds.invoke(args, vm),
        }
    }
}

#[derive(Clone, Trace, Finalize)]
pub enum AsynchronousFun<S, D> {
    Core(AsynchronousCoreFunction),
    StaticAsynchronous(S),
    Dynamic {
        ordinal: usize,
        fun: D,
    },
}

impl<S: PartialEq, D> PartialEq for AsynchronousFun<S, D> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AsynchronousFun::Core(a), AsynchronousFun::Core(b)) => a == b,
            (AsynchronousFun::StaticAsynchronous(a), AsynchronousFun::StaticAsynchronous(b)) => a == b,
            (AsynchronousFun::Dynamic {ordinal: a, ..}, AsynchronousFun::Dynamic {ordinal: b, ..}) => a == b,
            _ => false,
        }
    }
}

impl<S: Eq, D> Eq for AsynchronousFun<S, D> {}

impl<S: Ord, D> PartialOrd for AsynchronousFun<S, D> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<S: Ord, D> Ord for AsynchronousFun<S, D> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (AsynchronousFun::Core(a), AsynchronousFun::Core(b)) => a.cmp(b),
            (AsynchronousFun::Core(_), _) => Ordering::Less,
            (AsynchronousFun::StaticAsynchronous(a), AsynchronousFun::StaticAsynchronous(b)) => a.cmp(b),
            (AsynchronousFun::StaticAsynchronous(_), _) => Ordering::Less,
            (AsynchronousFun::Dynamic {ordinal: a, ..}, AsynchronousFun::Dynamic {ordinal: b, ..}) => a.cmp(b),
            (AsynchronousFun::Dynamic {..}, _) => Ordering::Greater,
        }
    }
}

impl<SS, SA, DS, DA, F, Fut> BuiltInAsyncFunction<V<SS, SA, DS, DA, F, Fut>, VvvmFailure<V<SS, SA, DS, DA, F, Fut>, F>, VvvmFuture<V<SS, SA, DS, DA, F, Fut>, Fut>> for AsynchronousFun<SA, DA> where
    SS: ValueBaseOrdered,
    DS: ValueBase,
    SA: ValueBaseOrdered + BuiltInAsyncFunction<V<SS, SA, DS, DA, F, Fut>, VvvmFailure<V<SS, SA, DS, DA, F, Fut>, F>, VvvmFuture<V<SS, SA, DS, DA, F, Fut>, Fut>>,
    DA: ValueBase + BuiltInAsyncFunction<V<SS, SA, DS, DA, F, Fut>, VvvmFailure<V<SS, SA, DS, DA, F, Fut>, F>, VvvmFuture<V<SS, SA, DS, DA, F, Fut>, Fut>>,
    F: 'static,
    Fut: 'static,
{
    fn arity(&self) -> Arity {
        match self {
            AsynchronousFun::Core(f) => BuiltInAsyncFunction::<V<SS, SA, DS, DA, F, Fut>, _, _>::arity(f),
            AsynchronousFun::StaticAsynchronous(sa) => sa.arity(),
            AsynchronousFun::Dynamic {fun: da, ..} => da.arity(),
        }
    }

    fn invoke(
        &mut self,
        args: &[V<SS, SA, DS, DA, F, Fut>],
        vm: &mut VirtualMachine<V<SS, SA, DS, DA, F, Fut>>,
    ) -> VvvmFuture<V<SS, SA, DS, DA, F, Fut>, Fut> {
        match self {
            AsynchronousFun::Core(f) => VvvmFuture::Core(BuiltInAsyncFunction::<V<SS, SA, DS, DA, F, Fut>, _, CoreFuture<V<SS, SA, DS, DA, F, Fut>>>::invoke(f, args, vm)),
            AsynchronousFun::StaticAsynchronous(sa) => sa.invoke(args, vm),
            AsynchronousFun::Dynamic {fun: da, ..} => da.invoke(args, vm),
        }
    }
}

#[derive(Clone, Trace, Finalize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AsynchronousCoreFunction {
    PreemptiveYield,
}

impl<SS, SA, DS, DA, F, Fut> BuiltInAsyncFunction<V<SS, SA, DS, DA, F, Fut>, VvvmFailure<V<SS, SA, DS, DA, F, Fut>, F>, CoreFuture<V<SS, SA, DS, DA, F, Fut>>> for AsynchronousCoreFunction where
    SS: ValueBaseOrdered,
    DS: ValueBase,
    SA: ValueBaseOrdered,
    DA: ValueBase,
    F: 'static,
    Fut: 'static,
{
    fn arity(&self) -> Arity {
        match self {
            AsynchronousCoreFunction::PreemptiveYield => 0,
        }
    }

    fn invoke(
        &mut self,
        _: &[V<SS, SA, DS, DA, F, Fut>],
        _: &mut VirtualMachine<V<SS, SA, DS, DA, F, Fut>>,
    ) -> CoreFuture<V<SS, SA, DS, DA, F, Fut>> {
        match self {
            AsynchronousCoreFunction::PreemptiveYield => ready(V::default()),
        }
    }
}

pub type CoreFuture<Val> = Ready<Val>;

#[derive(Clone, Trace, Finalize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SynchronousCoreFunction {
    ValueHalt,
    ValueTypeOf,
    ValueTruthy,
    ValueFalsey,

    OrderTotalCompare,
    OrderTotalLt,
    OrderTotalLeq,
    OrderTotalEq,
    OrderTotalGeq,
    OrderTotalGt,
    OrderTotalNeq,
    OrderTotalMin,
    OrderTotalMax,

    OrderPartialCompare,
    OrderPartialLt,
    OrderPartialLeq,
    OrderPartialEq,
    OrderPartialGeq,
    OrderPartialGt,
    OrderPartialNeq,
    OrderPartialGreatestLowerBound,
    OrderPartialLeastUpperBound,

    BoolNot,
    BoolAnd,
    BoolOr,
    BoolIf,
    BoolIff,
    BoolXor,

    FloatAdd,
    FloatSub,
    FloatMul,
    FloatDiv,
    FloatMulAdd,
    FloatNeg,
    FloatFloor,
    FloatCeil,
    FloatRound,
    FloatTrunc,
    FloatFract,
    FloatAbs,
    FloatSignum,
    FloatPow,
    FloatSqrt,
    FloatExp,
    FloatExp2,
    FloatLn,
    FloatLog2,
    FloatLog10,
    FloatHypot,
    FloatSin,
    FloatCos,
    FloatTan,
    FloatAsin,
    FloatAcos,
    FloatAtan,
    FloatAtan2,
    FloatExpM1,
    FloatLn1P,
    FloatSinh,
    FloatCosh,
    FloatTanh,
    FloatAsinh,
    FloatAcosh,
    FloatAtanh,
    FloatIsNormal,
    FloatToDegrees,
    FloatToRadians,
    FloatToInt,
    FloatFromInt,
    FloatToBits,
    FloatFromBits,

    IntSignum,
    IntAdd,
    IntSub,
    IntMul,
    IntDiv,
    IntDivTrunc,
    IntMod,
    IntModTrunc,
    IntNeg,
    IntAbs,
    IntPow,

    IntSatAdd,
    IntSatSub,
    IntSatMul,
    IntSatPow,

    IntWrapAdd,
    IntWrapSub,
    IntWrapMul,
    IntWrapDiv,
    IntWrapDivTrunc,
    IntWrapMod,
    IntWrapModTrunc,
    IntWrapNeg,
    IntWrapAbs,
    IntWrapPow,

    IntBitCountOnes,
    IntBitCountZeros,
    IntBitLeadingOnes,
    IntBitLeadingZeros,
    IntBitTrailingOnes,
    IntBitTrailingZeros,
    IntBitRotateLeft,
    IntBitRotateRight,
    IntBitReverseBytes,
    IntBitReverseBits,
    IntBitShl,
    IntBitShr,



}
use SynchronousCoreFunction::*;

impl<SS, SA, DS, DA, F, Fut> BuiltInSynchronousFunction<V<SS, SA, DS, DA, F, Fut>, CoreFailure<V<SS, SA, DS, DA, F, Fut>>> for SynchronousCoreFunction where
    SS: ValueBaseOrdered,
    DS: ValueBase,
    SA: ValueBaseOrdered,
    DA: ValueBase,
    F: 'static,
    Fut: 'static,
{
    fn arity(&self) -> Arity {
        match self {
            ValueHalt => 1,
            ValueTypeOf => 1,
            ValueTruthy => 1,
            ValueFalsey => 1,

            OrderTotalCompare => 2,
            OrderTotalLt => 2,
            OrderTotalLeq => 2,
            OrderTotalEq => 2,
            OrderTotalGeq => 2,
            OrderTotalGt => 2,
            OrderTotalNeq => 2,
            OrderTotalMin => 2,
            OrderTotalMax => 2,

            OrderPartialCompare => 2,
            OrderPartialLt => 2,
            OrderPartialLeq => 2,
            OrderPartialEq => 2,
            OrderPartialGeq => 2,
            OrderPartialGt => 2,
            OrderPartialNeq => 2,
            OrderPartialGreatestLowerBound => 2,
            OrderPartialLeastUpperBound => 2,

            BoolNot => 1,
            BoolAnd => 2,
            BoolOr => 2,
            BoolIf => 2,
            BoolIff => 2,
            BoolXor => 2,

            FloatAdd => 2,
            FloatSub => 2,
            FloatMul => 2,
            FloatDiv => 2,
            FloatMulAdd => 3,
            FloatNeg => 1,
            FloatFloor => 1,
            FloatCeil => 1,
            FloatRound => 1,
            FloatTrunc => 1,
            FloatFract => 1,
            FloatAbs => 1,
            FloatSignum => 1,
            FloatPow => 2,
            FloatSqrt => 1,
            FloatExp => 1,
            FloatExp2 => 1,
            FloatLn => 1,
            FloatLog2 => 1,
            FloatLog10 => 1,
            FloatHypot => 2,
            FloatSin => 1,
            FloatCos => 1,
            FloatTan => 1,
            FloatAsin => 1,
            FloatAcos => 1,
            FloatAtan => 1,
            FloatAtan2 => 2,
            FloatExpM1 => 1,
            FloatLn1P => 1,
            FloatSinh => 1,
            FloatCosh => 1,
            FloatTanh => 1,
            FloatAsinh => 1,
            FloatAcosh => 1,
            FloatAtanh => 1,
            FloatIsNormal => 1,
            FloatToDegrees => 1,
            FloatToRadians => 1,
            FloatToInt => 1,
            FloatFromInt => 1,
            FloatToBits => 1,
            FloatFromBits => 1,

            IntSignum => 1,
            IntAdd => 2,
            IntSub => 2,
            IntMul => 2,
            IntDiv => 2,
            IntDivTrunc => 2,
            IntMod => 2,
            IntModTrunc => 2,
            IntNeg => 1,
            IntAbs => 1,
            IntPow => 2,

            IntSatAdd => 2,
            IntSatSub => 2,
            IntSatMul => 2,
            IntSatPow => 2,

            IntWrapAdd => 2,
            IntWrapSub => 2,
            IntWrapMul => 2,
            IntWrapDiv => 2,
            IntWrapDivTrunc => 2,
            IntWrapMod => 2,
            IntWrapModTrunc => 2,
            IntWrapNeg => 1,
            IntWrapAbs => 1,
            IntWrapPow => 2,

            IntBitCountOnes => 1,
            IntBitCountZeros => 1,
            IntBitLeadingOnes => 1,
            IntBitLeadingZeros => 1,
            IntBitTrailingOnes => 1,
            IntBitTrailingZeros => 1,
            IntBitRotateLeft => 2,
            IntBitRotateRight => 2,
            IntBitReverseBytes => 1,
            IntBitReverseBits => 1,
            IntBitShl => 2,
            IntBitShr => 2,

            _ => unimplemented!(),
        }
    }

    fn invoke(
        &mut self,
        args: &[V<SS, SA, DS, DA, F, Fut>],
        vm: &mut VirtualMachine<V<SS, SA, DS, DA, F, Fut>>,
    ) -> Result<V<SS, SA, DS, DA, F, Fut>, CoreFailure<V<SS, SA, DS, DA, F, Fut>>> {
        match self {
            ValueHalt => value::halt(&args[0]),
            ValueTypeOf => value::type_of(&args[0]),
            ValueTruthy => value::truthy(&args[0]),
            ValueFalsey => value::falsey(&args[0]),

            OrderTotalCompare => order::total_compare(&args[0], &args[1]),
            OrderTotalLt => order::total_lt(&args[0], &args[1]),
            OrderTotalLeq => order::total_leq(&args[0], &args[1]),
            OrderTotalEq => order::total_eq(&args[0], &args[1]),
            OrderTotalGeq => order::total_geq(&args[0], &args[1]),
            OrderTotalGt => order::total_gt(&args[0], &args[1]),
            OrderTotalNeq => order::total_neq(&args[0], &args[1]),
            OrderTotalMin => order::total_min(&args[0], &args[1]),
            OrderTotalMax => order::total_max(&args[0], &args[1]),

            OrderPartialCompare => order::partial_compare(&args[0], &args[1]),
            OrderPartialLt => order::partial_lt(&args[0], &args[1]),
            OrderPartialLeq => order::partial_leq(&args[0], &args[1]),
            OrderPartialEq => order::partial_eq(&args[0], &args[1]),
            OrderPartialGeq => order::partial_geq(&args[0], &args[1]),
            OrderPartialGt => order::partial_gt(&args[0], &args[1]),
            OrderPartialNeq => order::partial_neq(&args[0], &args[1]),
            OrderPartialGreatestLowerBound => order::partial_greatest_lower_bound(&args[0], &args[1]),
            OrderPartialLeastUpperBound => order::partial_least_upper_bound(&args[0], &args[1]),

            BoolNot => boolean::not(&args[0]),
            BoolAnd => boolean::and(&args[0], &args[1]),
            BoolOr => boolean::or(&args[0], &args[1]),
            BoolIf => boolean::if_(&args[0], &args[1]),
            BoolIff => boolean::iff(&args[0], &args[1]),
            BoolXor => boolean::xor(&args[0], &args[1]),

            FloatAdd => float::add(&args[0], &args[1]),
            FloatSub => float::sub(&args[0], &args[1]),
            FloatMul => float::mul(&args[0], &args[1]),
            FloatDiv => float::div(&args[0], &args[1]),
            FloatMulAdd => float::mul_add(&args[0], &args[1], &args[2]),
            FloatNeg => float::neg(&args[0]),
            FloatFloor => float::floor(&args[0]),
            FloatCeil => float::ceil(&args[0]),
            FloatRound => float::round(&args[0]),
            FloatTrunc => float::trunc(&args[0]),
            FloatFract => float::fract(&args[0]),
            FloatAbs => float::abs(&args[0]),
            FloatSignum => float::signum(&args[0]),
            FloatPow => float::pow(&args[0], &args[1]),
            FloatSqrt => float::sqrt(&args[0]),
            FloatExp => float::exp(&args[0]),
            FloatExp2 => float::exp2(&args[0]),
            FloatLn => float::ln(&args[0]),
            FloatLog2 => float::log2(&args[0]),
            FloatLog10 => float::log10(&args[0]),
            FloatHypot => float::hypot(&args[0], &args[1]),
            FloatSin => float::sin(&args[0]),
            FloatCos => float::cos(&args[0]),
            FloatTan => float::tan(&args[0]),
            FloatAsin => float::asin(&args[0]),
            FloatAcos => float::acos(&args[0]),
            FloatAtan => float::atan(&args[0]),
            FloatAtan2 => float::atan2(&args[0], &args[1]),
            FloatExpM1 => float::exp_m1(&args[0]),
            FloatLn1P => float::ln_1p(&args[0]),
            FloatSinh => float::sinh(&args[0]),
            FloatCosh => float::cosh(&args[0]),
            FloatTanh => float::tanh(&args[0]),
            FloatAsinh => float::asinh(&args[0]),
            FloatAcosh => float::acosh(&args[0]),
            FloatAtanh => float::atanh(&args[0]),
            FloatIsNormal => float::is_normal(&args[0]),
            FloatToDegrees => float::to_degrees(&args[0]),
            FloatToRadians => float::to_radians(&args[0]),
            FloatToInt => float::to_int(&args[0]),
            FloatFromInt => float::from_int(&args[0]),
            FloatToBits => float::to_bits(&args[0]),
            FloatFromBits => float::from_bits(&args[0]),

            IntSignum => int::signum(&args[0]),
            IntAdd => int::add(&args[0], &args[1]),
            IntSub => int::sub(&args[0], &args[1]),
            IntMul => int::mul(&args[0], &args[1]),
            IntDiv => int::div(&args[0], &args[1]),
            IntDivTrunc => int::div_trunc(&args[0], &args[1]),
            IntMod => int::mod_(&args[0], &args[1]),
            IntModTrunc => int::mod_trunc(&args[0], &args[1]),
            IntNeg => int::neg(&args[0]),
            IntAbs => int::abs(&args[0]),
            IntPow => int::pow(&args[0], &args[1]),

            IntSatAdd => int::sat_add(&args[0], &args[1]),
            IntSatSub => int::sat_sub(&args[0], &args[1]),
            IntSatMul => int::sat_mul(&args[0], &args[1]),
            IntSatPow => int::sat_pow(&args[0], &args[1]),

            IntWrapAdd => int::wrap_add(&args[0], &args[1]),
            IntWrapSub => int::wrap_sub(&args[0], &args[1]),
            IntWrapMul => int::wrap_mul(&args[0], &args[1]),
            IntWrapDiv => int::wrap_div(&args[0], &args[1]),
            IntWrapDivTrunc => int::wrap_div_trunc(&args[0], &args[1]),
            IntWrapMod => int::wrap_mod(&args[0], &args[1]),
            IntWrapModTrunc => int::wrap_mod_trunc(&args[0], &args[1]),
            IntWrapNeg => int::wrap_neg(&args[0]),
            IntWrapAbs => int::wrap_abs(&args[0]),
            IntWrapPow => int::wrap_pow(&args[0], &args[1]),

            IntBitCountOnes => int::bit_count_ones(&args[0]),
            IntBitCountZeros => int::bit_count_zeros(&args[0]),
            IntBitLeadingOnes => int::bit_leading_ones(&args[0]),
            IntBitLeadingZeros => int::bit_leading_zeros(&args[0]),
            IntBitTrailingOnes => int::bit_trailing_ones(&args[0]),
            IntBitTrailingZeros => int::bit_trailing_zeros(&args[0]),
            IntBitRotateLeft => int::bit_rotate_left(&args[0], &args[1]),
            IntBitRotateRight => int::bit_rotate_right(&args[0], &args[1]),
            IntBitReverseBytes => int::bit_reverse_bytes(&args[0]),
            IntBitReverseBits => int::bit_reverse_bits(&args[0]),
            IntBitShl => int::bit_shl(&args[0], &args[1]),
            IntBitShr => int::bit_shr(&args[0], &args[1]),

            _ => unimplemented!(),
        }
    }
}

#[derive(Clone, Trace, Finalize, PartialEq, Eq, PartialOrd, Ord)]
pub enum DynamicCoreFunction {
    Foo,
}

impl<SS, SA, DS, DA, F, Fut> BuiltInSynchronousFunction<V<SS, SA, DS, DA, F, Fut>, CoreFailure<V<SS, SA, DS, DA, F, Fut>>> for DynamicCoreFunction where
    SS: ValueBaseOrdered,
    DS: ValueBase,
    SA: ValueBaseOrdered,
    DA: ValueBase,
    F: 'static,
    Fut: 'static,
{
    fn arity(&self) -> Arity {
        match self {
            _ => unimplemented!(),
        }
    }

    fn invoke(
        &mut self,
        args: &[V<SS, SA, DS, DA, F, Fut>],
        vm: &mut VirtualMachine<V<SS, SA, DS, DA, F, Fut>>,
    ) -> Result<V<SS, SA, DS, DA, F, Fut>, CoreFailure<V<SS, SA, DS, DA, F, Fut>>> {
        match self {
            _ => unimplemented!(),
        }
    }
}

pub enum CoreFailure<Val> {
    Halt(Val),
    NotBool(Val),
    NotFloat(Val),
    NotInt(Val),
    NotPositiveInt(Val),
    NotNonZeroInt(Val),
}
