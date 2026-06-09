use std::borrow::Cow;

use arrayvec::ArrayVec;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

const CAP: usize = 32;

pub trait DecimalExt {
    /// Converts the given decimal to stack-allocated ASCII byte vec. Removes
    /// the '.' and trims leading zeroes.
    ///
    /// I.e:
    /// - "45285.2" -> "452852"
    /// - "0.00100000" -> "100000"
    fn to_unscaled_array_vec(&self) -> ArrayVec<u8, CAP>;
}

impl DecimalExt for Decimal {
    fn to_unscaled_array_vec(&self) -> ArrayVec<u8, CAP> {
        let unpacked = self.unpack();
        let mut value = u128::from(unpacked.hi) << 64
            | u128::from(unpacked.mid) << 32
            | u128::from(unpacked.lo);

        // Digits are produced least-significant first, written back-to-front.
        let mut buf = [0u8; CAP];
        let mut pos = CAP;

        // Reduce the value until it fits in a u64 (almost always it already
        // does), where divisions are much cheaper than on the full 96-bit
        // mantissa.
        while value > u128::from(u64::MAX) {
            pos -= 1;
            buf[pos] = b'0' + (value % 10) as u8;
            value /= 10;
        }
        let mut value = value as u64;
        loop {
            pos -= 1;
            buf[pos] = b'0' + (value % 10) as u8;
            value /= 10;
            if value == 0 {
                break;
            }
        }

        let mut result = ArrayVec::new();
        result
            .try_extend_from_slice(&buf[pos..])
            .expect("a 96-bit mantissa has at most 29 digits");
        result
    }
}

#[must_use]
pub fn fmt_opt(value: Option<Decimal>) -> Cow<'static, str> {
    value.map_or_else(
        || Cow::Borrowed("NaN"),
        |value| Cow::Owned(value.to_string()),
    )
}

#[must_use]
pub fn fmt_pct_opt(value: Option<Decimal>) -> Cow<'static, str> {
    value.map_or(Cow::Borrowed("NaN"), |x| {
        Cow::Owned(format!("{}%", (x * dec!(100)).round_dp(2)))
    })
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;
    use test_case::test_case;

    use super::*;

    #[test_case(dec!(45285.2), b"452852")]
    #[test_case(dec!(0.00100000), b"100000")]
    #[test_case(dec!(45285.20), b"4528520")]
    #[test_case(dec!(101.00100000), b"10100100000")]
    #[test_case(dec!(0.001), b"1")]
    #[test_case(dec!(0), b"0")]
    #[test_case(Decimal::MAX, b"79228162514264337593543950335")]
    fn to_unscaled_array_vec(input: Decimal, expected_output: &[u8]) {
        assert_eq!(input.to_unscaled_array_vec().as_slice(), expected_output);
    }
}
