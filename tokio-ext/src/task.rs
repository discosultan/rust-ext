/// A set of [`tokio::task::JoinHandle`]s similar to [`tokio::task::JoinSet`],
/// but with a key difference: tasks must be spawned separately before their
/// handles can be added to this set.
///
/// This struct provides a way to manage multiple asynchronous tasks by
/// collecting their [`tokio::task::JoinHandle`]s. Unlike
/// [`tokio::task::JoinSet`], which spawns tasks and adds their handles
/// directly, this `JoinSet` requires you to spawn tasks externally and then
/// insert the resulting handles into the set manually.
#[derive(Default)]
#[cfg(feature = "rt")]
pub struct JoinSet<T> {
    handles: Vec<tokio::task::JoinHandle<T>>,
}

#[cfg(feature = "rt")]
impl<T> JoinSet<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            handles: Vec::new(),
        }
    }

    pub fn insert(&mut self, handle: tokio::task::JoinHandle<T>) {
        self.handles.push(handle);
    }

    pub async fn join_all(mut self) -> Vec<Result<T, tokio::task::JoinError>> {
        let handles = std::mem::take(&mut self.handles);
        futures_util::future::join_all(handles).await
    }

    pub async fn try_join_all(mut self) -> Result<Vec<T>, tokio::task::JoinError> {
        let handles = std::mem::take(&mut self.handles);
        futures_util::future::try_join_all(handles).await
    }

    pub fn drain(&mut self) -> Vec<tokio::task::JoinHandle<T>> {
        std::mem::take(&mut self.handles)
    }
}

#[cfg(feature = "rt")]
impl<T> Drop for JoinSet<T> {
    fn drop(&mut self) {
        for handle in &self.handles {
            handle.abort();
        }
    }
}

#[cfg(feature = "rt")]
impl<T> FromIterator<tokio::task::JoinHandle<T>> for JoinSet<T> {
    fn from_iter<I: IntoIterator<Item = tokio::task::JoinHandle<T>>>(iter: I) -> Self {
        Self {
            handles: iter.into_iter().collect(),
        }
    }
}

/// # Panics
///
/// This method panics if called outside of a Tokio runtime.
#[cfg(tokio_unstable)]
#[cfg(feature = "rt")]
#[track_caller]
pub fn spawn_named<F>(name: &str, future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    tokio::task::Builder::new()
        .name(name)
        .spawn(future)
        .unwrap()
}

#[cfg(not(tokio_unstable))]
#[cfg(feature = "rt")]
#[track_caller]
pub fn spawn_named<F>(_name: &str, future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    tokio::task::spawn(future)
}

#[cfg(test)]
#[cfg(feature = "rt")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn join_set_join_all() {
        let mut join_set = JoinSet::new();

        // Spawn and insert tasks.
        for i in 0..3 {
            join_set.insert(tokio::spawn(async move { i }));
        }

        // Await all tasks.
        let results = join_set.join_all().await;

        assert_eq!(
            results.into_iter().map(Result::unwrap).collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
    }
}
