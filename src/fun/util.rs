#![macro_use]

use crate::{V, ValueBaseOrdered, ValueBase, VvvmFailure, VvvmFuture};
use super::CoreFailure;

pub type R<SS, SA, DS, DA, F, Fut> = Result<V<SS, SA, DS, DA, F, Fut>, CoreFailure<V<SS, SA, DS, DA, F, Fut>>>;

macro_rules! fun {
    ($name:ident ($($arg:ident),*) $body:block) => (
        pub fn $name <SS, SA, DS, DA, F, Fut>($($arg: &V<SS, SA, DS, DA, F, Fut>),*) -> R<SS, SA, DS, DA, F, Fut> where
            SS: ValueBaseOrdered,
            DS: ValueBase,
            SA: ValueBaseOrdered,
            DA: ValueBase,
            F: 'static,
            Fut: 'static,
        {
            $body
        }
    );
}

pub fn as_bool<SS, SA, DS, DA, F, Fut>(v: &V<SS, SA, DS, DA, F, Fut>) -> Result<bool, CoreFailure<V<SS, SA, DS, DA, F, Fut>>> where
    SS: ValueBaseOrdered,
    DS: ValueBase,
    SA: ValueBaseOrdered,
    DA: ValueBase,
    F: 'static,
    Fut: 'static,
{
    match v {
        V::Bool(b) => Ok(b.clone()),
        _ => Err(CoreFailure::NotBool(v.clone())),
    }
}

pub fn as_float<SS, SA, DS, DA, F, Fut>(v: &V<SS, SA, DS, DA, F, Fut>) -> Result<f64, CoreFailure<V<SS, SA, DS, DA, F, Fut>>> where
    SS: ValueBaseOrdered,
    DS: ValueBase,
    SA: ValueBaseOrdered,
    DA: ValueBase,
    F: 'static,
    Fut: 'static,
{
    match v {
        V::Float(f) => Ok(f.0.clone()),
        _ => Err(CoreFailure::NotFloat(v.clone())),
    }
}

pub fn as_int<SS, SA, DS, DA, F, Fut>(v: &V<SS, SA, DS, DA, F, Fut>) -> Result<i64, CoreFailure<V<SS, SA, DS, DA, F, Fut>>> where
    SS: ValueBaseOrdered,
    DS: ValueBase,
    SA: ValueBaseOrdered,
    DA: ValueBase,
    F: 'static,
    Fut: 'static,
{
    match v {
        V::Int(n) => Ok(n.clone()),
        _ => Err(CoreFailure::NotInt(v.clone())),
    }
}
