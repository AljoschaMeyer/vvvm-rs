use core::cmp::Ordering;

#[derive(Clone, Copy)]
pub struct PavoFloat(pub f64);

impl PartialEq for PavoFloat {
    fn eq(&self, other: &Self) -> bool {
        if self.0.is_nan() && other.0.is_nan() {
            true
        } else {
            self.0.to_bits() == other.0.to_bits()
        }
    }
}

impl Eq for PavoFloat {}

impl PartialOrd for PavoFloat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PavoFloat {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.0.is_nan() && !other.0.is_nan() {
            Ordering::Less
        } else if !self.0.is_nan() && other.0.is_nan() {
            Ordering::Greater
        } else if self.0.is_sign_negative() && !self.0.is_sign_negative() {
            Ordering::Less
        } else if !self.0.is_sign_negative() && self.0.is_sign_negative() {
            Ordering::Greater
        } else {
            self.0.partial_cmp(&other.0).unwrap()
        }
    }
}
