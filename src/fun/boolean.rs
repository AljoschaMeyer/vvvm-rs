use super::util::*;
use crate::{V, ValueBaseOrdered, ValueBase};

fun!(not(b) {
    Ok(V::boo(!as_bool(b)?))
});

fun!(and(b, c) {
    Ok(V::boo(as_bool(b)? && as_bool(c)?))
});

fun!(or(b, c) {
    Ok(V::boo(as_bool(b)? || as_bool(c)?))
});

fun!(if_(b, c) {
    let b = as_bool(b)?;
    let c = as_bool(c)?;
    if b {
        Ok(V::boo(c))
    } else {
        Ok(V::boo(true))
    }
});

fun!(iff(b, c) {
    Ok(V::boo(as_bool(b)? == as_bool(c)?))
});

fun!(xor(b, c) {
    Ok(V::boo(as_bool(b)? != as_bool(c)?))
});
