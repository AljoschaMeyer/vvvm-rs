use guvm_rs::Value;

use super::util::*;
use super::CoreFailure;
use crate::{V, ValueBaseOrdered, ValueBase};

fun!(halt(v) {
    Err(CoreFailure::Halt(v.clone()))
});

fun!(type_of(v) {
    match v {
        V::Nil => Ok(V::string("nil")),
        _ => Ok(V::string("foo")),
    }
});

fun!(truthy(v) {
    Ok(V::boo(v.truthy()))
});

fun!(falsey(v) {
    Ok(V::boo(!v.truthy()))
});
