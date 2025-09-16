pub trait ResultExt<T, E1, E2> {
    fn flatten_into<E3>(self) -> Result<T, E3>
    where
        E3: From<E1> + From<E2>;
}

impl<T, E1, E2> ResultExt<T, E1, E2> for Result<Result<T, E1>, E2> {
    fn flatten_into<E3>(self) -> Result<T, E3>
    where
        E3: From<E1> + From<E2>,
    {
        match self {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(e)) => Err(E3::from(e)),
            Err(e) => Err(E3::from(e)),
        }
    }
}
