use crate::loops::{First, Loop, Next, Task};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

pub struct SerialLoop<M, E, F, I, U, H, S, O> {
    model: RefCell<M>,
    taskq: Cell<VecDeque<Task<E, F>>>,
    init_fn: Option<I>,
    update_fn: U,
    handle_fn: H,
    source_fn: Option<S>,
    observe_fn: Option<O>,
}

impl<M, E, F, I, U, H, S, O> Loop for SerialLoop<M, E, F, I, U, H, S, O> {
    type Model = M;
    type Event = E;
    type Effect = F;

    fn current(&self) -> &Self::Model {
        unimplemented!()
    }

    fn dispatch(&self, event: Self::Event) -> &Self {
        let mut taskq = self.taskq.take();
        taskq.push_back(Task::Event(event));
        self.taskq.replace(taskq);
        self
    }
}

impl<M, E, F, I, U, H, S, O> SerialLoop<M, E, F, I, U, H, S, O>
where
    I: Fn(&M) -> First<M, F>,
    U: Fn(&M, E) -> Next<M, F>,
    H: Fn(&M, F) -> Vec<E>,
    S: Fn(&M) -> Vec<E>,
    O: Fn(&M),
{
    pub fn new(
        model: M,
        init_fn: Option<I>,
        update_fn: U,
        handle_fn: H,
        source_fn: Option<S>,
        observe_fn: Option<O>,
    ) -> Self {
        SerialLoop {
            model: RefCell::new(model),
            taskq: Cell::new(VecDeque::new()),
            init_fn,
            update_fn,
            handle_fn,
            source_fn,
            observe_fn,
        }
    }

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
