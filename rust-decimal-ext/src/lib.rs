use std::borrow::Cow;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

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
