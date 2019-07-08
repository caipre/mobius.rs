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
    pub fn first(model: M, effects: &[F]) -> First<M, F> {
        let effects = effects.to_vec();
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
    pub fn next(model: M, effects: &[F]) -> Self {
        let model = Some(model);
        let effects = effects.to_vec();
        Next { model, effects }
    }

    pub fn dispatch(effect: F) -> Self {
        let model = None;
        let effects = vec![effect];
        Next { model, effects }
    }

    pub fn dispatch_vec(effects: Vec<F>) -> Self {
        let model = None;
        let effects = effects.to_vec();
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

pub struct LoopBuilder<M, E, F, U, H, O> {
    model: PhantomData<M>,
    event: PhantomData<E>,
    effect: PhantomData<F>,
    //    init_fn: Option<Box<I>>,
    update_fn: U,
    handle_fn: H,
    //    source_fn: Option<Box<S>>,
    observe_fn: Option<Box<O>>,
}

impl<M, E, F, U, H, O> LoopBuilder<M, E, F, U, H, O>
where
    //    I: Fn(&M) -> First<M, F>,
    U: Fn(&M, E) -> Next<M, F>,
    H: Fn(&M, F) -> Vec<E>,
    //    S: Fn(&M) -> Vec<E>,
    O: Fn(&M),
{
    pub fn new(update_fn: U, handle_fn: H) -> Self {
        LoopBuilder {
            model: PhantomData,
            event: PhantomData,
            effect: PhantomData,
            //            init_fn: None,
            update_fn,
            handle_fn,
            //            source_fn: None,
            observe_fn: None,
        }
    }

    //    pub fn init(&mut self, func: I) -> &mut Self {
    //        self.init_fn = Some(Box::new(func));
    //        self
    //    }

    //    pub fn source(&mut self, func: S) -> &mut Self {
    //        self.source_fn = Some(Box::new(func));
    //        self
    //    }

    pub fn observe(mut self, func: O) -> Self {
        self.observe_fn = Some(Box::new(func));
        self
    }

    pub fn start(self, model: M) -> SingleThreadedLoop<M, E, F, U, H, O> {
        SingleThreadedLoop::new(model, self.update_fn, self.handle_fn, self.observe_fn)
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

pub struct SingleThreadedLoop<M, E, F, U, H, O> {
    model: RefCell<M>,
    taskq: Cell<VecDeque<Task<E, F>>>,
    //    init_fn: Option<Box<I>>,
    update_fn: U,
    handle_fn: H,
    //    source_fn: Option<Box<S>>,
    observe_fn: Option<Box<O>>,
}

impl<M, E, F, U, H, O> SingleThreadedLoop<M, E, F, U, H, O>
where
    //    I: Fn(&M) -> First<M, F>,
    U: Fn(&M, E) -> Next<M, F>,
    H: Fn(&M, F) -> Vec<E>,
    //    S: Fn(&M) -> Vec<E>,
    O: Fn(&M),
{
    pub fn dispatch(&self, event: E) -> &Self {
        let mut taskq = self.taskq.take();
        taskq.push_back(Task::Event(event));
        self.taskq.replace(taskq);
        self
    }

    //

    fn new(model: M, update_fn: U, handle_fn: H, observe_fn: Option<Box<O>>) -> Self {
        SingleThreadedLoop {
            model: RefCell::new(model),
            taskq: Cell::new(VecDeque::new()),
            //            init_fn: self.init_fn,
            update_fn,
            handle_fn,
            //            source_fn: self.source_fn,
            observe_fn,
        }
    }

    pub fn run(&self) -> &Self {
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
