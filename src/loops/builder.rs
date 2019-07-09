use crate::loops::serial::SerialLoop;
use crate::loops::{First, Next};
use std::marker::PhantomData;

pub struct Builder<M, E, F, I, U, H, S, O> {
    model: PhantomData<M>,
    event: PhantomData<E>,
    effect: PhantomData<F>,
    init_fn: Option<I>,
    update_fn: U,
    handle_fn: H,
    source_fn: Option<S>,
    observe_fn: Option<O>,
}

type InitFn<M, F> = fn(&M) -> First<M, F>;
type SourceFn<M, E> = fn(&M) -> Vec<E>;
type ObserveFn<M> = fn(&M);

impl<M, E, F, U, H> Builder<M, E, F, InitFn<M, F>, U, H, SourceFn<M, E>, ObserveFn<M>>
where
    U: Fn(&M, E) -> Next<M, F>,
    H: Fn(&M, F) -> Vec<E>,
{
    /// Return a Builder configured with the supplied update and handle functions.
    pub fn new(update_fn: U, handle_fn: H) -> Self {
        Builder {
            model: PhantomData,
            event: PhantomData,
            effect: PhantomData,
            init_fn: None,
            update_fn,
            handle_fn,
            source_fn: None,
            observe_fn: None,
        }
    }
}

impl<M, E, F, I, U, H, S, O> Builder<M, E, F, I, U, H, S, O>
where
    I: Fn(&M) -> First<M, F>,
{
    /// Return a Builder configured with the supplied init function while
    /// retaining the same values for all other fields.
    pub fn init<F0: Fn(&M) -> First<M, F>>(self, func: F0) -> Builder<M, E, F, F0, U, H, S, O> {
        Builder {
            model: self.model,
            event: self.event,
            effect: self.effect,
            init_fn: Some(func),
            update_fn: self.update_fn,
            handle_fn: self.handle_fn,
            source_fn: self.source_fn,
            observe_fn: self.observe_fn,
        }
    }
}

impl<M, E, F, I, U, H, S, O> Builder<M, E, F, I, U, H, S, O>
where
    S: Fn(&M) -> Vec<E>,
{
    /// Return a Builder configured with the supplied source function while
    /// retaining the same values for all other fields.
    pub fn source<F0: Fn(&M) -> Vec<E>>(self, func: F0) -> Builder<M, E, F, I, U, H, F0, O> {
        Builder {
            model: self.model,
            event: self.event,
            effect: self.effect,
            init_fn: self.init_fn,
            update_fn: self.update_fn,
            handle_fn: self.handle_fn,
            source_fn: Some(func),
            observe_fn: self.observe_fn,
        }
    }
}

impl<M, E, F, I, U, H, S, O> Builder<M, E, F, I, U, H, S, O>
where
    O: Fn(&M),
{
    /// Return a Builder configured with the supplied observe function while
    /// retaining the same values for all other fields.
    pub fn observe<F0: Fn(&M)>(self, func: F0) -> Builder<M, E, F, I, U, H, S, F0> {
        Builder {
            model: self.model,
            event: self.event,
            effect: self.effect,
            init_fn: self.init_fn,
            update_fn: self.update_fn,
            handle_fn: self.handle_fn,
            source_fn: self.source_fn,
            observe_fn: Some(func),
        }
    }
}

impl<M, E, F, I, U, H, S, O> Builder<M, E, F, I, U, H, S, O>
where
    I: Fn(&M) -> First<M, F>,
    U: Fn(&M, E) -> Next<M, F>,
    H: Fn(&M, F) -> Vec<E>,
    S: Fn(&M) -> Vec<E>,
    O: Fn(&M),
{
    pub fn start(self, model: M) -> SerialLoop<M, E, F, I, U, H, S, O> {
        SerialLoop::new(
            model,
            self.init_fn,
            self.update_fn,
            self.handle_fn,
            self.source_fn,
            self.observe_fn,
        )
    }
}
