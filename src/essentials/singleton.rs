/// Enables use of `static`s as global variables.
///
/// Usage:
/// ```
/// let mut inner = unsafe { SINGLETON.take() };
/// inner.method();
/// unsafe { SINGLETON.give(inner); }
/// ```
pub struct Singleton<T> {
    pub inner: Option<T>,
}

impl<T> Singleton<T> {
    /// Takes ownership of singleton's inner value.
    /// If the inner value is already taken, program crashes.
    pub fn take(&mut self) -> T {
        let p = core::mem::replace(&mut self.inner, None);
        // TODO: better double take handling
        p.unwrap()
    }

    /// Returns taken value from singleton to the singleton
    /// for next use.
    pub fn give(&mut self, inner: T) {
        let _ = core::mem::replace(&mut self.inner, Some(inner));
    }
}
