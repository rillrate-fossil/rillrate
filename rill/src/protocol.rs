pub mod internal {
    use meio::prelude::Action;

    /// Wrapper for internal communications.
    pub struct Internal<T: Send + 'static>(pub T);

    impl<T: Send + 'static> Action for Internal<T> {}
}
