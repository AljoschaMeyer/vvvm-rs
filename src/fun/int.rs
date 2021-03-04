use super::util::*;
use crate::{V, ValueBaseOrdered, ValueBase};

fun!(signum(n) {
    let n = as_int(n)?;
    Ok(V::int(n.signum()))
});

fun!(add(n, m) {
    let n = as_int(n)?;
    let m = as_int(m)?;
    match n.checked_add(m) {
        Some(yay) => Ok(V::int(yay)),
        None => Ok(V::err_nil()),
    }
});

fun!(sub(n, m) {
    let n = as_int(n)?;
    let m = as_int(m)?;
    match n.checked_sub(m) {
        Some(yay) => Ok(V::int(yay)),
        None => Ok(V::err_nil()),
    }
});

fun!(mul(n, m) {
    let n = as_int(n)?;
    let m = as_int(m)?;
    match n.checked_add(m) {
        Some(yay) => Ok(V::int(yay)),
        None => Ok(V::err_nil()),
    }
});

fun!(div(n, m) {
    let n = as_int(n)?;
    let m = as_non_zero_int(m)?;
    match n.checked_div_euclid(m) {
        Some(yay) => Ok(V::int(yay)),
        None => Ok(V::err_nil()),
    }
});

fun!(div_trunc(n, m) {
    let n = as_int(n)?;
    let m = as_non_zero_int(m)?;
    match n.checked_div(m) {
        Some(yay) => Ok(V::int(yay)),
        None => Ok(V::err_nil()),
    }
});

fun!(mod_(n, m) {
    let n = as_int(n)?;
    let m = as_non_zero_int(m)?;
    match n.checked_rem_euclid(m) {
        Some(yay) => Ok(V::int(yay)),
        None => Ok(V::err_nil()),
    }
});

fun!(mod_trunc(n, m) {
    let n = as_int(n)?;
    let m = as_non_zero_int(m)?;
    match n.checked_rem(m) {
        Some(yay) => Ok(V::int(yay)),
        None => Ok(V::err_nil()),
    }
});

fun!(neg(n) {
    let n = as_int(n)?;

    match n.checked_neg() {
        Some(yay) => Ok(V::int(yay)),
        None => Ok(V::err_nil()),
    }
});

fun!(abs(n) {
    let n = as_int(n)?;

    match n.checked_abs() {
        Some(yay) => Ok(V::int(yay)),
        None => Ok(V::err_nil()),
    }
});

fun!(pow(n, m) {
    let n = as_int(n)?;
    let m = as_positive_int(m)?;
    match n.checked_pow(m as u32) {
        Some(yay) => Ok(V::int(yay)),
        None => Ok(V::err_nil()),
    }
});

fun!(sat_add(n, m) {
    let n = as_int(n)?;
    let m = as_int(m)?;
    Ok(V::int(n.saturating_add(m)))
});

fun!(sat_sub(n, m) {
    let n = as_int(n)?;
    let m = as_int(m)?;
    Ok(V::int(n.saturating_sub(m)))
});

fun!(sat_mul(n, m) {
    let n = as_int(n)?;
    let m = as_int(m)?;
    Ok(V::int(n.saturating_mul(m)))
});

fun!(sat_pow(n, m) {
    let n = as_int(n)?;
    let m = as_positive_int(m)?;
    Ok(V::int(n.saturating_pow(m as u32)))
});

fun!(wrap_add(n, m) {
    let n = as_int(n)?;
    let m = as_int(m)?;
    Ok(V::int(n.wrapping_add(m)))
});

fun!(wrap_sub(n, m) {
    let n = as_int(n)?;
    let m = as_int(m)?;
    Ok(V::int(n.wrapping_sub(m)))
});

fun!(wrap_mul(n, m) {
    let n = as_int(n)?;
    let m = as_int(m)?;
    Ok(V::int(n.wrapping_mul(m)))
});

fun!(wrap_div(n, m) {
    let n = as_int(n)?;
    let m = as_non_zero_int(m)?;
    Ok(V::int(n.wrapping_div_euclid(m)))
});

fun!(wrap_div_trunc(n, m) {
    let n = as_int(n)?;
    let m = as_non_zero_int(m)?;
    Ok(V::int(n.wrapping_div(m)))
});

fun!(wrap_mod(n, m) {
    let n = as_int(n)?;
    let m = as_non_zero_int(m)?;
    Ok(V::int(n.wrapping_rem_euclid(m)))
});

fun!(wrap_mod_trunc(n, m) {
    let n = as_int(n)?;
    let m = as_non_zero_int(m)?;
    Ok(V::int(n.wrapping_rem(m)))
});

fun!(wrap_neg(n) {
    let n = as_int(n)?;
    Ok(V::int(n.wrapping_neg()))
});

fun!(wrap_abs(n) {
    let n = as_int(n)?;
    Ok(V::int(n.wrapping_abs()))
});

fun!(wrap_pow(n, m) {
    let n = as_int(n)?;
    let m = as_positive_int(m)?;
    Ok(V::int(n.wrapping_pow(m as u32)))
});

fun!(bit_count_ones(n) {
    let n = as_int(n)?;
    Ok(V::int(n.count_ones() as i64))
});

fun!(bit_count_zeros(n) {
    let n = as_int(n)?;
    Ok(V::int(n.count_zeros() as i64))
});

fun!(bit_leading_ones(n) {
    let n = as_int(n)?;
    Ok(V::int(n.leading_ones() as i64))
});

fun!(bit_leading_zeros(n) {
    let n = as_int(n)?;
    Ok(V::int(n.leading_zeros() as i64))
});

fun!(bit_trailing_ones(n) {
    let n = as_int(n)?;
    Ok(V::int(n.trailing_ones() as i64))
});

fun!(bit_trailing_zeros(n) {
    let n = as_int(n)?;
    Ok(V::int(n.trailing_zeros() as i64))
});

fun!(bit_rotate_left(n, m) {
    let n = as_int(n)?;
    let m = as_positive_int(m)?;
    Ok(V::int(n.rotate_left(m as u32)))
});

fun!(bit_rotate_right(n, m) {
    let n = as_int(n)?;
    let m = as_positive_int(m)?;
    Ok(V::int(n.rotate_right(m as u32)))
});

fun!(bit_reverse_bytes(n) {
    let n = as_int(n)?;
    Ok(V::int(n.swap_bytes()))
});

fun!(bit_reverse_bits(n) {
    let n = as_int(n)?;
    Ok(V::int(n.reverse_bits()))
});

fun!(bit_shl(n, m) {
    let n = as_int(n)?;
    let m = as_positive_int(m)?;
    if m >= 64 {
        Ok(V::int(0))
    } else {
        Ok(V::int(n << m))
    }
});

fun!(bit_shr(n, m) {
    let n = as_int(n)?;
    let m = as_positive_int(m)?;
    if m >= 64 {
        Ok(V::int(0))
    } else {
        Ok(V::int(n >> m))
    }
});
