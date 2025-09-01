use std::{
    fmt,
    str::FromStr,
    time::{Duration, UNIX_EPOCH},
};

use serde::{
    Deserialize, Deserializer,
    de::{self, Visitor},
};

/// Custom deserialization function that uses [`FromStr`].
pub fn from_str<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    struct StringVisitor<T>(std::marker::PhantomData<T>);

    impl<T> Visitor<'_> for StringVisitor<T>
    where
        T: FromStr,
        T::Err: fmt::Display,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("A string that can be parsed into the requested type.")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            T::from_str(value).map_err(E::custom)
        }
    }

    deserializer.deserialize_str(StringVisitor(std::marker::PhantomData))
}

/// Custom deserialization function that uses [`FromStr`]. If deserialization
/// fails, returns the default value.
pub fn from_str_or_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + Default,
    T::Err: fmt::Display,
{
    struct StringVisitor<T>(std::marker::PhantomData<T>);

    impl<T> Visitor<'_> for StringVisitor<T>
    where
        T: FromStr + Default,
        T::Err: fmt::Display,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("A string that can be parsed into the requested type.")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(T::from_str(value).unwrap_or_default())
        }
    }

    deserializer.deserialize_str(StringVisitor(std::marker::PhantomData))
}

/// For example, deserializes the string "2023-09-22T10:33:05.709993Z" into a
/// duration since epoch.
pub fn duration_iso_8601<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringVisitor;

    impl Visitor<'_> for StringVisitor {
        type Value = Duration;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("A string that can be parsed into the requested type.")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let system_time = humantime::parse_rfc3339_weak(value).map_err(E::custom)?;
            system_time.duration_since(UNIX_EPOCH).map_err(E::custom)
        }
    }

    deserializer.deserialize_str(StringVisitor)
}

pub fn duration_from_secs<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let secs: u64 = Deserialize::deserialize(deserializer)?;
    Ok(Duration::from_secs(secs))
}

pub fn duration_from_millis<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let millis: u64 = Deserialize::deserialize(deserializer)?;
    Ok(Duration::from_millis(millis))
}

pub fn duration_from_nanos<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let nanos: u64 = Deserialize::deserialize(deserializer)?;
    Ok(Duration::from_nanos(nanos))
}

/// Custom deserialization function that uses [`FromStr`] for optional fields.
pub fn from_str_opt<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    struct OptStringVisitor<T>(std::marker::PhantomData<T>);

    impl<T> Visitor<'_> for OptStringVisitor<T>
    where
        T: FromStr,
        T::Err: fmt::Display,
    {
        type Value = Option<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("A null or a string that can be parsed into the requested type.")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            T::from_str(value).map(Some).map_err(E::custom)
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    }

    deserializer.deserialize_any(OptStringVisitor(std::marker::PhantomData))
}

/// For example, deserializes the string "1hour 12min 5s" into a duration.
pub fn duration_humantime<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringVisitor;

    impl Visitor<'_> for StringVisitor {
        type Value = Duration;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("A string that can be parsed into the requested type.")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            humantime::parse_duration(value).map_err(E::custom)
        }
    }

    deserializer.deserialize_str(StringVisitor)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use test_case::test_case;

    use super::*;

    #[test]
    fn test_deserialize_from_str() {
        #[derive(Deserialize)]
        struct Number(#[serde(deserialize_with = "from_str")] u32);

        let output: Number = serde_json::from_str("\"1\"").unwrap();
        assert_eq!(output.0, 1);
    }

    #[test]
    fn test_deserialize_from_str_streaming() {
        #[derive(Deserialize)]
        struct Number(#[serde(deserialize_with = "from_str")] u32);

        let input = b"\"1\"";
        // Deserialize from a Reader (streaming). Streaming cannot hand out
        // &'a str that borrow from the input.
        let cursor = Cursor::new(&input[..]);
        let output: Number = serde_json::from_reader(cursor).unwrap();
        assert_eq!(output.0, 1);
    }

    #[test_case("\"1\"" => 1)]
    #[test_case("\"a\"" => 0)]
    fn test_deserialize_from_str_or_default(input: &str) -> u32 {
        #[derive(Deserialize)]
        struct Number(#[serde(deserialize_with = "from_str_or_default")] u32);

        let output: Number = serde_json::from_str(input).unwrap();
        output.0
    }

    #[test]
    fn test_deserialize_from_str_or_default_streaming() {
        #[derive(Deserialize)]
        struct Number(#[serde(deserialize_with = "from_str_or_default")] u32);

        let input = b"\"1\"";
        // Deserialize from a Reader (streaming). Streaming cannot hand out
        // &'a str that borrow from the input.
        let cursor = Cursor::new(&input[..]);
        let output: Number = serde_json::from_reader(cursor).unwrap();
        assert_eq!(output.0, 1);
    }

    #[test]
    fn test_deserialize_duration_iso_8601() {
        #[derive(Deserialize)]
        struct Iso8601Duration(#[serde(deserialize_with = "duration_iso_8601")] Duration);

        let output: Iso8601Duration =
            serde_json::from_str("\"2023-10-06T17:35:55.440295Z\"").unwrap();
        assert_eq!(output.0, Duration::new(1_696_613_755, 440_295_000));
    }

    #[test]
    fn test_deserialize_duration_from_millis() {
        #[derive(Deserialize)]
        struct MillisDuration(#[serde(deserialize_with = "duration_from_millis")] Duration);

        // 2020-06-30 03:24:41.683
        let output: MillisDuration = serde_json::from_str("1593487481683").unwrap();
        assert_eq!(output.0, Duration::new(1_593_487_481, 683_000_000));
    }

    #[test]
    fn test_deserialize_duration_from_nanos() {
        #[derive(Deserialize)]
        struct NanosDuration(#[serde(deserialize_with = "duration_from_nanos")] Duration);

        // 2020-06-30 03:24:41.683297666
        let output: NanosDuration = serde_json::from_str("1593487481683297666").unwrap();
        assert_eq!(output.0, Duration::new(1_593_487_481, 683_297_666));
    }

    #[test]
    fn test_deserialize_duration_humantime() {
        #[derive(Deserialize)]
        struct HumantimeDuration(#[serde(deserialize_with = "duration_humantime")] Duration);

        let output: HumantimeDuration = serde_json::from_str("\"1hour 12min 5s\"").unwrap();
        assert_eq!(output.0, Duration::new(4325, 0));
    }
}
