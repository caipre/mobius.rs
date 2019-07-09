pub mod builder;

mod serial;

/// A Mobius loop.
pub trait Loop {
    type Model;
    type Event;
    type Effect;

    fn current(&self) -> &Self::Model;
    fn dispatch(&self, event: Self::Event) -> &Self;
}

/// Represents the intended _first_ state of a Mobius [Loop].
pub struct First<M, F> {
    model: M,
    effects: Vec<F>,
}

impl<M, F> First<M, F>
where
    F: Clone,
{
    pub fn first(model: M, effects: Vec<F>) -> First<M, F> {
        First { model, effects }
    }
}

impl<M, F> From<M> for First<M, F> {
    fn from(model: M) -> Self {
        let effects = vec![];
        First { model, effects }
    }
}

/// Represents the intended _next_ state of a Mobius [Loop].
pub struct Next<M, F> {
    model: Option<M>,
    effects: Vec<F>,
}

impl<M, F> Next<M, F>
where
    F: Clone,
{
    pub fn next(model: M, effects: Vec<F>) -> Self {
        let model = Some(model);
        Next { model, effects }
    }

    pub fn dispatch(effect: F) -> Self {
        let model = None;
        let effects = vec![effect];
        Next { model, effects }
    }

    pub fn dispatch_vec(effects: Vec<F>) -> Self {
        let model = None;
        Next { model, effects }
    }

    pub fn no_change() -> Self {
        let model = None;
        let effects = vec![];
        Next { model, effects }
    }
}

impl<M, F> From<M> for Next<M, F> {
    fn from(model: M) -> Self {
        let model = Some(model);
        let effects = vec![];
        Next { model, effects }
    }
}

enum Task<E, F> {
    Event(E),
    Effect(F),
}
