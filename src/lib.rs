use core::cmp::Ordering;

use gc::{Gc, GcCell, Trace, Finalize, custom_trace};
use gc_derive::{Trace, Finalize};

use guvm_rs::{Value, BuiltInAsyncFunction, BuiltInSynchronousFunction, Closure};
use gc_immutable_collections::{Array, Map};

mod float;
use float::PavoFloat;

mod fun;
use fun::{Fun, SynchronousFun, AsynchronousFun, CoreFailure, CoreFuture};

#[derive(Finalize)]
pub enum V<SS, SA, DS, DA, F, Fut>
where
    SS: ValueBaseOrdered,
    DS: ValueBase,
    SA: ValueBaseOrdered,
    DA: ValueBase,
    F: 'static,
    Fut: 'static,
{
    Nil,
    Bool(bool),
    Float(PavoFloat),
    Int(i64),
    Array(Array<Self>),
    Map(Map<Self, Self>),
    Fun(Fun<SS, SA, DS, DA, F, Fut>),
}

impl<SS, SA, DS, DA, F, Fut> Clone for V<SS, SA, DS, DA, F, Fut>
where
    SS: ValueBaseOrdered,
    DS: ValueBase,
    SA: ValueBaseOrdered,
    DA: ValueBase,
{
    fn clone(&self) -> Self {
        match self {
            V::Nil => V::Nil,
            V::Bool(b) => V::Bool(b.clone()),
            V::Float(f) => V::Float(f.clone()),
            V::Int(i) => V::Int(i.clone()),
            V::Array(a) => V::Array(a.clone()),
            V::Map(m) => V::Map(m.clone()),
            V::Fun(f) => V::Fun(f.clone()),
        }
    }
}

unsafe impl<SS, SA, DS, DA, F, Fut> Trace for V<SS, SA, DS, DA, F, Fut>
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
            V::Nil | V::Bool(_) | V::Float(_) | V::Int(_) => {}
            V::Array(a) => mark(a),
            V::Map(m) => mark(m),
            V::Fun(f) => mark(f),
        }
    });
}

impl<SS, SA, DS, DA, F, Fut> Default for V<SS, SA, DS, DA, F, Fut>
where
    SS: ValueBaseOrdered,
    DS: ValueBase,
    SA: ValueBaseOrdered,
    DA: ValueBase,
    F: 'static,
    Fut: 'static,
{
    fn default() -> Self {
        V::Nil
    }
}

impl<SS, SA, DS, DA, F, Fut> PartialEq for V<SS, SA, DS, DA, F, Fut>
where
    SS: ValueBaseOrdered,
    DS: ValueBase,
    SA: ValueBaseOrdered,
    DA: ValueBase,
    F: 'static,
    Fut: 'static,
{
    fn eq(&self, other: &Self) -> bool {
        unimplemented!()
    }
}

impl<SS, SA, DS, DA, F, Fut> Eq for V<SS, SA, DS, DA, F, Fut>
where
    SS: ValueBaseOrdered,
    DS: ValueBase,
    SA: ValueBaseOrdered,
    DA: ValueBase,
    F: 'static,
    Fut: 'static,
{}

impl<SS, SA, DS, DA, F, Fut> PartialOrd for V<SS, SA, DS, DA, F, Fut>
where
    SS: ValueBaseOrdered,
    DS: ValueBase,
    SA: ValueBaseOrdered,
    DA: ValueBase,
    F: 'static,
    Fut: 'static,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        unimplemented!()
    }
}

impl<SS, SA, DS, DA, F, Fut> Ord for V<SS, SA, DS, DA, F, Fut>
where
    SS: ValueBaseOrdered,
    DS: ValueBase,
    SA: ValueBaseOrdered,
    DA: ValueBase,
    F: 'static,
    Fut: 'static,
{
    fn cmp(&self, other: &Self) -> Ordering {
        unimplemented!()
    }
}

impl<SS, SA, DS, DA, F, Fut> Value for V<SS, SA, DS, DA, F, Fut>
where
    SS: ValueBaseOrdered,
    DS: ValueBase,
    SA: ValueBaseOrdered,
    DA: ValueBase,
    F: 'static,
    Fut: 'static,
{
    type Failure = VvvmFailure<Self, F>;
    type Fut = VvvmFuture<Self, Fut>;
    type BuiltInFunction = SynchronousFun<SS, DS>;
    type BuiltInAsync = AsynchronousFun<SA, DA>;

    fn truthy(&self) -> bool {
        match self {
            V::Nil | V::Bool(false) => false,
            _ => true,
        }
    }

    fn as_built_in_function(self) -> Option<Self::BuiltInFunction> {
        match self {
            V::Fun(Fun::SynchronousFunction(f)) => Some(f),
            _ => None,
        }
    }
    fn as_built_in_function_ref(&self) -> Option<&Self::BuiltInFunction> {
        match self {
            V::Fun(Fun::SynchronousFunction(f)) => Some(f),
            _ => None,
        }
    }
    fn as_built_in_function_mut(&mut self) -> Option<&mut Self::BuiltInFunction> {
        match self {
            V::Fun(Fun::SynchronousFunction(f)) => Some(f),
            _ => None,
        }
    }

    fn as_built_in_async(self) -> Option<Self::BuiltInAsync> {
        match self {
            V::Fun(Fun::AsynchronousFunction(f)) => Some(f),
            _ => None,
        }
    }
    fn as_built_in_async_ref(&self) -> Option<&Self::BuiltInAsync> {
        match self {
            V::Fun(Fun::AsynchronousFunction(f)) => Some(f),
            _ => None,
        }
    }
    fn as_built_in_async_mut(&mut self) -> Option<&mut Self::BuiltInAsync> {
        match self {
            V::Fun(Fun::AsynchronousFunction(f)) => Some(f),
            _ => None,
        }
    }

    fn new_closure(f: Closure<Self>) -> Self {
        V::Fun(Fun::Closure(f))
    }
    fn as_closure(self) -> Option<Closure<Self>> {
        match self {
            V::Fun(Fun::Closure(f)) => Some(f),
            _ => None,
        }
    }
    fn as_closure_ref(&self) -> Option<&Closure<Self>> {
        match self {
            V::Fun(Fun::Closure(f)) => Some(f),
            _ => None,
        }
    }
    fn as_closure_mut(&mut self) -> Option<&mut Closure<Self>> {
        match self {
            V::Fun(Fun::Closure(f)) => Some(f),
            _ => None,
        }
    }
}

impl<SS, SA, DS, DA, F, Fut> V<SS, SA, DS, DA, F, Fut>
where
    SS: ValueBaseOrdered,
    DS: ValueBase,
    SA: ValueBaseOrdered,
    DA: ValueBase,
    F: 'static,
    Fut: 'static,
{
    pub fn nil() -> Self {
        V::Nil
    }

    pub fn boo(b: bool) -> Self {
        V::Bool(b)
    }

    pub fn string(s: &str) -> Self {
        unimplemented!()
    }

    pub fn ok(v: Self) -> Self {
        unimplemented!()
    }

    pub fn err(v: Self) -> Self {
        unimplemented!()
    }

    pub fn err_nil() -> Self {
        Self::err(Self::nil())
    }

    pub fn partial_compare(&self, other: &Self) -> Option<Ordering> {
        unimplemented!()
    }

    pub fn partial_lt(&self, other: &Self) -> Option<bool> {
        unimplemented!()
    }

    pub fn partial_leq(&self, other: &Self) -> Option<bool> {
        unimplemented!()
    }

    pub fn partial_eq(&self, other: &Self) -> Option<bool> {
        unimplemented!()
    }

    pub fn partial_geq(&self, other: &Self) -> Option<bool> {
        unimplemented!()
    }

    pub fn partial_gt(&self, other: &Self) -> Option<bool> {
        unimplemented!()
    }

    pub fn partial_neq(&self, other: &Self) -> Option<bool> {
        unimplemented!()
    }

    pub fn partial_greatest_lower_bound(&self, other: &Self) -> Option<Self> {
        unimplemented!()
    }

    pub fn partial_least_upper_bound(&self, other: &Self) -> Option<Self> {
        unimplemented!()
    }
}

pub enum VvvmFailure<Val, F> {
    Core(CoreFailure<Val>),
    Other(F),
}

impl<Val, F> From<CoreFailure<Val>> for VvvmFailure<Val, F> {
    fn from(f: CoreFailure<Val>) -> Self {
        VvvmFailure::Core(f)
    }
}

pub enum VvvmFuture<Val, Fut> {
    Core(CoreFuture<Val>),
    Other(Fut),
}

pub trait ValueBase: Sized + Trace + Finalize + Clone + Default + 'static {}

pub trait ValueBaseOrdered: ValueBase + PartialEq + Eq + PartialOrd + Ord {}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
