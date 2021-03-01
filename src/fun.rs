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
mod boolean;

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

    ValueTotalCompare,
    ValueTotalLt,
    ValueTotalLeq,
    ValueTotalEq,
    ValueTotalGeq,
    ValueTotalGt,
    ValueTotalNeq,
    ValueTotalMin,
    ValueTotalMax,

    ValuePartialCompare,
    ValuePartialLt,
    ValuePartialLeq,
    ValuePartialEq,
    ValuePartialGeq,
    ValuePartialGt,
    ValuePartialNeq,
    ValuePartialGreatestLowerBound,
    ValuePartialLeastUpperBound,

    BoolNot,
    BoolAnd,
    BoolOr,
    BoolIf,
    BoolIff,
    BoolXor,

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

            ValueTotalCompare => 2,
            ValueTotalLt => 2,
            ValueTotalLeq => 2,
            ValueTotalEq => 2,
            ValueTotalGeq => 2,
            ValueTotalGt => 2,
            ValueTotalNeq => 2,
            ValueTotalMin => 2,
            ValueTotalMax => 2,

            ValuePartialCompare => 2,
            ValuePartialLt => 2,
            ValuePartialLeq => 2,
            ValuePartialEq => 2,
            ValuePartialGeq => 2,
            ValuePartialGt => 2,
            ValuePartialNeq => 2,
            ValuePartialGreatestLowerBound => 2,
            ValuePartialLeastUpperBound => 2,

            BoolNot => 1,
            BoolAnd => 2,
            BoolOr => 2,
            BoolIf => 2,
            BoolIff => 2,
            BoolXor => 2,

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

            ValueTotalCompare => value::total_compare(&args[0], &args[1]),
            ValueTotalLt => value::total_lt(&args[0], &args[1]),
            ValueTotalLeq => value::total_leq(&args[0], &args[1]),
            ValueTotalEq => value::total_eq(&args[0], &args[1]),
            ValueTotalGeq => value::total_geq(&args[0], &args[1]),
            ValueTotalGt => value::total_gt(&args[0], &args[1]),
            ValueTotalNeq => value::total_neq(&args[0], &args[1]),
            ValueTotalMin => value::total_min(&args[0], &args[1]),
            ValueTotalMax => value::total_max(&args[0], &args[1]),

            ValuePartialCompare => value::partial_compare(&args[0], &args[1]),
            ValuePartialLt => value::partial_lt(&args[0], &args[1]),
            ValuePartialLeq => value::partial_leq(&args[0], &args[1]),
            ValuePartialEq => value::partial_eq(&args[0], &args[1]),
            ValuePartialGeq => value::partial_geq(&args[0], &args[1]),
            ValuePartialGt => value::partial_gt(&args[0], &args[1]),
            ValuePartialNeq => value::partial_neq(&args[0], &args[1]),
            ValuePartialGreatestLowerBound => value::partial_greatest_lower_bound(&args[0], &args[1]),
            ValuePartialLeastUpperBound => value::partial_least_upper_bound(&args[0], &args[1]),

            BoolNot => boolean::not(&args[0]),
            BoolAnd => boolean::and(&args[0], &args[1]),
            BoolOr => boolean::or(&args[0], &args[1]),
            BoolIf => boolean::if_(&args[0], &args[1]),
            BoolIff => boolean::iff(&args[0], &args[1]),
            BoolXor => boolean::xor(&args[0], &args[1]),

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
}
