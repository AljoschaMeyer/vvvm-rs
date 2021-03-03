use core::cmp::Ordering;

use guvm_rs::Value;

use super::util::*;
use super::CoreFailure;
use crate::{V, ValueBaseOrdered, ValueBase};

fun!(total_compare(v, w) {
    Ok(match v.cmp(w) {
        Ordering::Less => V::string("<"),
        Ordering::Equal => V::string("="),
        Ordering::Greater => V::string(">"),
    })
});

fun!(total_lt(v, w) {
    Ok(V::boo(v < w))
});

fun!(total_leq(v, w) {
    Ok(V::boo(v <= w))
});

fun!(total_eq(v, w) {
    Ok(V::boo(v == w))
});

fun!(total_geq(v, w) {
    Ok(V::boo(v >= w))
});

fun!(total_gt(v, w) {
    Ok(V::boo(v > w))
});

fun!(total_neq(v, w) {
    Ok(V::boo(v != w))
});

fun!(total_min(v, w) {
    Ok(core::cmp::min(v, w).clone())
});

fun!(total_max(v, w) {
    Ok(core::cmp::max(v, w).clone())
});

fun!(partial_compare(v, w) {
    match v.partial_compare(w) {
        Some(Ordering::Less) => Ok(V::ok(V::string("<"))),
        Some(Ordering::Equal) => Ok(V::ok(V::string("="))),
        Some(Ordering::Greater) => Ok(V::ok(V::string(">"))),
        None => Ok(V::err_nil()),
    }
});

fun!(partial_lt(v, w) {
    match v.partial_lt(w) {
        Some(b) => Ok(V::ok(V::boo(b))),
        None => Ok(V::err_nil()),
    }
});

fun!(partial_leq(v, w) {
    match v.partial_leq(w) {
        Some(b) => Ok(V::ok(V::boo(b))),
        None => Ok(V::err_nil()),
    }
});

fun!(partial_eq(v, w) {
    match v.partial_eq(w) {
        Some(b) => Ok(V::ok(V::boo(b))),
        None => Ok(V::err_nil()),
    }
});

fun!(partial_geq(v, w) {
    match v.partial_geq(w) {
        Some(b) => Ok(V::ok(V::boo(b))),
        None => Ok(V::err_nil()),
    }
});

fun!(partial_gt(v, w) {
    match v.partial_gt(w) {
        Some(b) => Ok(V::ok(V::boo(b))),
        None => Ok(V::err_nil()),
    }
});

fun!(partial_neq(v, w) {
    match v.partial_neq(w) {
        Some(b) => Ok(V::ok(V::boo(b))),
        None => Ok(V::err_nil()),
    }
});

fun!(partial_greatest_lower_bound(v, w) {
    match v.partial_greatest_lower_bound(w) {
        Some(b) => Ok(V::ok(b)),
        None => Ok(V::err_nil()),
    }
});

fun!(partial_least_upper_bound(v, w) {
    match v.partial_least_upper_bound(w) {
        Some(b) => Ok(V::ok(b)),
        None => Ok(V::err_nil()),
    }
});
