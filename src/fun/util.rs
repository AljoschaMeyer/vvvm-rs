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
