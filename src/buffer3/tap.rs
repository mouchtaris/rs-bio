use std::borrow::Borrow;

impl<S> Tap for S {}

pub trait Tap: Sized {
    fn tap(self, block: impl FnOnce(&Self)) -> Self {
        block(&self);
        self
    }

    fn tap_ok<T, E>(self, block: impl FnOnce(&T)) -> Self
    where
        Self: Borrow<Result<T, E>>,
    {
        if let Ok(t) = self.borrow().as_ref() {
            block(t);
        }
        self
    }
}
