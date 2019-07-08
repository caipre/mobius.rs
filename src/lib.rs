use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::marker::PhantomData;

/// Represents the first state of a mobius loop.
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

///// Represents the desired next state for a mobius loop.
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

pub struct LoopBuilder<M, E, F, I, U, H, S, O> {
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

impl<M, E, F, U, H> LoopBuilder<M, E, F, InitFn<M, F>, U, H, SourceFn<M, E>, ObserveFn<M>>
where
    U: Fn(&M, E) -> Next<M, F>,
    H: Fn(&M, F) -> Vec<E>,
{
    pub fn new(update_fn: U, handle_fn: H) -> Self {
        LoopBuilder {
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

impl<M, E, F, I, U, H, S, O> LoopBuilder<M, E, F, I, U, H, S, O>
where
    I: Fn(&M) -> First<M, F>,
{
    /// Return a new Builder configured with the supplied init function while
    /// retaining the same values for all other fields.
    pub fn init<F0: Fn(&M) -> First<M, F>>(self, func: F0) -> LoopBuilder<M, E, F, F0, U, H, S, O> {
        LoopBuilder {
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

impl<M, E, F, I, U, H, S, O> LoopBuilder<M, E, F, I, U, H, S, O>
where
    S: Fn(&M) -> Vec<E>,
{
    /// Return a new Builder configured with the supplied source function while
    /// retaining the same values for all other fields.
    pub fn source<F0: Fn(&M) -> Vec<E>>(self, func: F0) -> LoopBuilder<M, E, F, I, U, H, F0, O> {
        LoopBuilder {
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

impl<M, E, F, I, U, H, S, O> LoopBuilder<M, E, F, I, U, H, S, O>
where
    O: Fn(&M),
{
    /// Return a new Builder configured with the supplied observe function while
    /// retaining the same values for all other fields.
    pub fn observe<F0: Fn(&M)>(self, func: F0) -> LoopBuilder<M, E, F, I, U, H, S, F0> {
        LoopBuilder {
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

impl<M, E, F, I, U, H, S, O> LoopBuilder<M, E, F, I, U, H, S, O>
where
    I: Fn(&M) -> First<M, F>,
    U: Fn(&M, E) -> Next<M, F>,
    H: Fn(&M, F) -> Vec<E>,
    S: Fn(&M) -> Vec<E>,
    O: Fn(&M),
{
    pub fn start(self, model: M) -> SingleThreadedLoop<M, E, F, I, U, H, S, O> {
        SingleThreadedLoop {
            model: RefCell::new(model),
            taskq: Cell::new(VecDeque::new()),
            init_fn: self.init_fn,
            update_fn: self.update_fn,
            handle_fn: self.handle_fn,
            source_fn: self.source_fn,
            observe_fn: self.observe_fn,
        }
    }
}

enum Task<E, F> {
    Event(E),
    Effect(F),
}

pub trait Loop<M, E, F> {
    fn current(&self) -> M;
    fn dispatch(&self, event: E) -> &Self;
}

pub struct SingleThreadedLoop<M, E, F, I, U, H, S, O> {
    model: RefCell<M>,
    taskq: Cell<VecDeque<Task<E, F>>>,
    init_fn: Option<I>,
    update_fn: U,
    handle_fn: H,
    source_fn: Option<S>,
    observe_fn: Option<O>,
}

impl<M, E, F, I, U, H, S, O> SingleThreadedLoop<M, E, F, I, U, H, S, O>
where
    I: Fn(&M) -> First<M, F>,
    U: Fn(&M, E) -> Next<M, F>,
    H: Fn(&M, F) -> Vec<E>,
    S: Fn(&M) -> Vec<E>,
    O: Fn(&M),
{
    pub fn dispatch(&self, event: E) -> &Self {
        let mut taskq = self.taskq.take();
        taskq.push_back(Task::Event(event));
        self.taskq.replace(taskq);
        self
    }

    //

    pub fn run(&self) -> &Self {
        self.init();
        loop {
            let mut taskq = self.taskq.take();
            match taskq.pop_front() {
                Some(task) => {
                    let tasks = self.handle(task);
                    taskq.append(&mut tasks.into());
                    self.taskq.replace(taskq);
                }
                None => break,
            }
        }
        self
    }

    //

    fn init(&self) {
        if let Some(ref init_fn) = self.init_fn {
            let first = (init_fn)(&self.model.borrow());
            self.model.replace(first.model);
            let tasks = first
                .effects
                .into_iter()
                .map(|e| Task::Effect(e))
                .collect::<Vec<_>>();
            let mut taskq = self.taskq.take();
            taskq.append(&mut tasks.into());
            self.taskq.replace(taskq);
        }
    }

    fn handle(&self, task: Task<E, F>) -> Vec<Task<E, F>> {
        match task {
            Task::Event(event) => {
                let effects = self.update(event);
                effects.into_iter().map(|e| Task::Effect(e)).collect()
            }
            Task::Effect(effect) => {
                let events = (self.handle_fn)(&self.model.borrow(), effect);
                events.into_iter().map(|e| Task::Event(e)).collect()
            }
        }
    }

    fn update(&self, event: E) -> Vec<F> {
        let next = (self.update_fn)(&self.model.borrow(), event);
        if let Some(next_model) = next.model {
            self.model.replace(next_model);
            if let Some(ref func) = self.observe_fn {
                (func)(&self.model.borrow());
            }
        }
        next.effects
    }
}
