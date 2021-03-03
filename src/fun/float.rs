use super::util::*;
use crate::{V, ValueBaseOrdered, ValueBase};

fun!(add(x, y) {
    let x = as_float(x)?;
    let y = as_float(y)?;
    Ok(V::float(x + y))
});

fun!(sub(x, y) {
    let x = as_float(x)?;
    let y = as_float(y)?;
    Ok(V::float(x - y))
});

fun!(mul(x, y) {
    let x = as_float(x)?;
    let y = as_float(y)?;
    Ok(V::float(x * y))
});

fun!(div(x, y) {
    let x = as_float(x)?;
    let y = as_float(y)?;
    Ok(V::float(x / y))
});

fun!(mul_add(x, y, z) {
    let x = as_float(x)?;
    let y = as_float(y)?;
    let z = as_float(z)?;
    Ok(V::float(x.mul_add(y, z)))
});

fun!(neg(x) {
    let x = as_float(x)?;
    Ok(V::float(-x))
});

fun!(floor(x) {
    let x = as_float(x)?;
    Ok(V::float(x.floor()))
});

fun!(ceil(x) {
    let x = as_float(x)?;
    Ok(V::float(x.ceil()))
});

fun!(round(x) {
    let x = as_float(x)?;
    Ok(V::float(x.round()))
});

fun!(trunc(x) {
    let x = as_float(x)?;
    Ok(V::float(x.trunc()))
});

fun!(fract(x) {
    let x = as_float(x)?;
    Ok(V::float(x.fract()))
});

fun!(abs(x) {
    let x = as_float(x)?;
    Ok(V::float(x.abs()))
});

fun!(signum(x) {
    let x = as_float(x)?;
    Ok(V::float(x.signum()))
});

fun!(pow(x, y) {
    let x = as_float(x)?;
    let y = as_float(y)?;
    Ok(V::float(x.powf(y)))
});

fun!(sqrt(x) {
    let x = as_float(x)?;
    Ok(V::float(x.sqrt()))
});

fun!(exp(x) {
    let x = as_float(x)?;
    Ok(V::float(x.exp()))
});

fun!(exp2(x) {
    let x = as_float(x)?;
    Ok(V::float(x.exp2()))
});

fun!(ln(x) {
    let x = as_float(x)?;
    Ok(V::float(x.ln()))
});

fun!(log2(x) {
    let x = as_float(x)?;
    Ok(V::float(x.log2()))
});

fun!(log10(x) {
    let x = as_float(x)?;
    Ok(V::float(x.log10()))
});

fun!(hypot(x, y) {
    let x = as_float(x)?;
    let y = as_float(y)?;
    Ok(V::float(x.hypot(y)))
});

fun!(sin(x) {
    let x = as_float(x)?;
    Ok(V::float(x.sin()))
});

fun!(cos(x) {
    let x = as_float(x)?;
    Ok(V::float(x.cos()))
});

fun!(tan(x) {
    let x = as_float(x)?;
    Ok(V::float(x.tan()))
});

fun!(asin(x) {
    let x = as_float(x)?;
    Ok(V::float(x.asin()))
});

fun!(acos(x) {
    let x = as_float(x)?;
    Ok(V::float(x.acos()))
});

fun!(atan(x) {
    let x = as_float(x)?;
    Ok(V::float(x.atan()))
});

fun!(atan2(x, y) {
    let x = as_float(x)?;
    let y = as_float(y)?;
    Ok(V::float(x.atan2(y)))
});

fun!(exp_m1(x) {
    let x = as_float(x)?;
    Ok(V::float(x.exp_m1()))
});

fun!(ln_1p(x) {
    let x = as_float(x)?;
    Ok(V::float(x.ln_1p()))
});

fun!(sinh(x) {
    let x = as_float(x)?;
    Ok(V::float(x.sinh()))
});

fun!(cosh(x) {
    let x = as_float(x)?;
    Ok(V::float(x.cosh()))
});

fun!(tanh(x) {
    let x = as_float(x)?;
    Ok(V::float(x.tanh()))
});

fun!(asinh(x) {
    let x = as_float(x)?;
    Ok(V::float(x.asinh()))
});

fun!(acosh(x) {
    let x = as_float(x)?;
    Ok(V::float(x.acosh()))
});

fun!(atanh(x) {
    let x = as_float(x)?;
    Ok(V::float(x.atanh()))
});

fun!(is_normal(x) {
    let x = as_float(x)?;
    Ok(V::boo(x.is_normal()))
});

fun!(to_degrees(x) {
    let x = as_float(x)?;
    Ok(V::float(x.to_degrees()))
});

fun!(to_radians(x) {
    let x = as_float(x)?;
    Ok(V::float(x.to_radians()))
});

fun!(to_int(xf) {
    let x = as_float(xf)?;

    if x >= (std::i64::MAX as f64) || x <= (std::i64::MIN as f64) || x.is_nan() {
        Ok(V::err(xf.clone()))
    } else {
        return Ok(V::int(x as i64));
    }
});

fun!(from_int(n) {
    let n = as_int(n)?;
    Ok(V::float(n as f64))
});

fun!(to_bits(x) {
    let x = as_float(x)?;
    if x.is_nan() {
        Ok(V::int(-1))
    } else {
        Ok(V::int(x.to_bits() as i64))
    }
});

fun!(from_bits(n) {
    let n = as_int(n)?;
    Ok(V::float(f64::from_bits(n as u64)))
});
