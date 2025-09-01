use std::str::FromStr;

/// An opinionated parsing for comma separated values into a boxed slice.
///
/// For example, useful when parsing arguments with clap. `Box<[T]>` is used
/// instead of `Vec<T>` due to https://github.com/clap-rs/clap/issues/4808.
pub fn parse_comma_separated_boxed_slice<T>(values: &str) -> Result<Box<[T]>, T::Err>
where
    T: FromStr,
{
    values.split(',').map(|s| s.trim().parse()).collect()
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case("42", &[42] ; "single value")]
    #[test_case("1,2,3,4,5", &[1, 2, 3, 4, 5] ; "multiple values")]
    #[test_case("1, 2 , 3,  4  ,5", &[1, 2, 3, 4, 5] ; "values with whitespace")]
    fn test_parse_comma_separated_boxed_slice_valid(input: &str, expected: &[u32]) {
        let result = parse_comma_separated_boxed_slice::<u32>(input);
        assert_eq!(result.unwrap().as_ref(), expected);
    }

    #[test_case("" ; "empty string")]
    #[test_case("1,2,abc,4" ; "contains non-numeric")]
    #[test_case("1,2,3.5,4" ; "contains float")]
    #[test_case("1,-2,3" ; "contains negative for u32")]
    #[test_case("1,2,3," ; "trailing comma")]
    #[test_case(",1,2,3" ; "leading comma")]
    #[test_case("1,,3" ; "double comma")]
    fn test_parse_comma_separated_boxed_slice_invalid(input: &str) {
        let result = parse_comma_separated_boxed_slice::<u32>(input);
        assert!(result.is_err());
    }
}
