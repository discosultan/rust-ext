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
        // Uses a similar implementation as the Decimal Display impl.
        let unpacked = self.unpack();
        let mut working: [u32; 3] = [unpacked.lo, unpacked.mid, unpacked.hi];

        if working == [0, 0, 0] {
            return ArrayVec::from_iter([b'0']);
        }

        let mut result = ArrayVec::new();
        let mut temp_chars = [0u8; CAP];
        let mut char_count = 0;

        while working != [0, 0, 0] {
            let mut remainder = 0u64;
            for part in working.iter_mut().rev() {
                remainder = remainder * (1u64 << 32) + u64::from(*part);
                *part = (remainder / 10) as u32;
                remainder %= 10;
            }
            temp_chars[char_count] = b'0' + remainder as u8;
            char_count += 1;
        }

        for i in 0..char_count {
            result.push(temp_chars[char_count - 1 - i]);
        }

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
    fn to_unscaled_array_vec(input: Decimal, expected_output: &[u8]) {
        assert_eq!(input.to_unscaled_array_vec().as_slice(), expected_output);
    }
}
