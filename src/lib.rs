use crossbeam_channel::{Receiver, Sender};
use rayon::{ThreadPool, ThreadPoolBuilder};

/// A function for initializing a loop from a given model.
type InitFn<M, F> = Box<dyn Fn(&M) -> First<M, F>>;

/// A function that drives the loop forward with a new model and effects.
type UpdateFn<M, E, F> = Box<dyn Fn(&M, E) -> Next<M, F>>;

/// Responsible for holding and updating the model.
pub struct Store<M, E, F> {
    model: M,
    init_fn: InitFn<M, F>,
    update_fn: UpdateFn<M, E, F>,
}

impl<M, E, F> Store<M, E, F> {
    pub fn new(model: M, init_fn: InitFn<M, F>, update_fn: UpdateFn<M, E, F>) -> Self {
        Store {
            model,
            init_fn,
            update_fn,
        }
    }

    fn init(&mut self) -> First<&M, F> {
        let first = (self.init_fn)(&self.model);
        self.model = first.model;
        First::from(&self.model)
    }

    fn update(&mut self, event: E) -> Vec<F> {
        let next = (self.update_fn)(&self.model, event);
        if let Some(model) = next.model {
            self.model = model;
        }
        next.effects
    }
}

/// Represents the initial state of a mobius loop.
pub struct First<M, F> {
    pub model: M,
    pub effects: Vec<F>,
}

impl<M, F> First<M, F> {
    pub fn of(model: M, effects: Vec<F>) -> First<M, F> {
        First { model, effects }
    }

    pub fn from(model: M) -> First<M, F> {
        First {
            model,
            effects: vec![],
        }
    }
}

pub struct Next<M, F> {
    model: Option<M>,
    effects: Vec<F>,
}

impl<M, F> Next<M, F> {
    pub fn of(model: M, effects: Vec<F>) -> Next<M, F> {
        Next {
            model: Some(model),
            effects,
        }
    }

    pub fn from(model: M) -> Next<M, F> {
        Next {
            model: Some(model),
            effects: vec![],
        }
    }

    pub fn dispatch(effects: Vec<F>) -> Next<M, F> {
        Next {
            model: None,
            effects,
        }
    }

    pub fn pass() -> Next<M, F> {
        Next {
            model: None,
            effects: vec![],
        }
    }
}

/// A function to connect to a loop.
type Connectable<I, O> = Box<dyn Fn(Receiver<O>) -> Sender<I>>;

pub struct Loop<M, E, F> {
    store: Store<M, E, F>,
    effecthandler: Connectable<F, E>,
    eventsource: Connectable<M, E>,
    threadpool: ThreadPool,
    sender: Sender<F>,
    receiver: Receiver<E>,
}

impl<M, E, F> Loop<M, E, F> {
    pub fn new(
        store: Store<M, E, F>,
        effecthandler: Connectable<F, E>,
        eventsource: Connectable<M, E>,
        threadpool: ThreadPool,
    ) -> Self {
        let (esender, ereceiver) = crossbeam_channel::unbounded();
        let (fsender, freceiver) = crossbeam_channel::unbounded();
        Loop {
            store,
            effecthandler,
            eventsource,
            threadpool,
            sender: fsender,
            receiver: ereceiver,
        }
    }

    pub fn start_from(&mut self, model: M) {
        self.store.model = model;
    }

    pub fn dispatch(&mut self, event: E) {
        let fs = self.store.update(event);
        (self.effecthandler)(self.receiver.clone());
    }
}
