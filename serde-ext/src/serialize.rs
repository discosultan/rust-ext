use std::fmt::Display;

use serde::{
    Serialize, Serializer,
    ser::{SerializeSeq, SerializeTuple},
};

/// Custom serialization function to support serializing const generic arrays.
///
/// <https://github.com/serde-rs/serde/issues/1937#issuecomment-812461429>
pub fn const_generic_array<S: Serializer, T: Serialize, const N: usize>(
    data: &[T; N],
    ser: S,
) -> Result<S::Ok, S::Error> {
    let mut s = ser.serialize_tuple(N)?;
    for item in data {
        s.serialize_element(item)?;
    }
    s.end()
}

/// Custom serialization function that uses [`Display`].
pub fn to_string<S>(value: &impl Display, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.collect_str(value)
}

/// Custom serialization function that uses [`Display`].
pub fn to_string_opt<S>(value: &Option<impl Display>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(value) = value {
        serializer.collect_str(&value)
    } else {
        serializer.serialize_none()
    }
}

/// Custom serialization function that uses [`Display`].
pub fn slice_elements_to_string<S>(
    values: &[impl Display],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    /// Wrapper that serializes using [`Display`].
    struct DisplayWrapper<'a, T: Display>(&'a T);

    impl<'a, T: Display> Serialize for DisplayWrapper<'a, T> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.collect_str(self.0)
        }
    }

    let mut seq = serializer.serialize_seq(Some(values.len()))?;
    for value in values {
        seq.serialize_element(&DisplayWrapper(value))?;
    }
    seq.end()
}
