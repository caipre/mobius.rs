use mobius::*;
use rayon::ThreadPoolBuilder;

type Model = u64;

#[derive(Debug)]
enum Event {
    Increment,
    Decrement,
}

#[derive(Debug)]
enum Effect {
    PrintOut(&'static str),
    PrintError(&'static str),
}

fn main() {
    let init_fn = Box::new(|m: &Model| First::<Model, Effect>::from(0));
    let update_fn = Box::new(|m: &Model, e: Event| {
        dbg!(m, &e);
        match e {
            Event::Increment => Next::from(m + 1),
            Event::Decrement => {
                if *m == 0 {
                    Next::dispatch(vec![Effect::PrintError("negative")])
                } else {
                    Next::from(m - 1)
                }
            }
        }
    });

    let (fsender, freceiver) = crossbeam_channel::unbounded();
    let effecthandler = Box::new(move |ereceiver| fsender.clone());

    let (esender, ereceiver) = crossbeam_channel::unbounded();
    let eventsource = Box::new(move |freceiver| esender.clone());

    let threadpool = ThreadPoolBuilder::default().build().unwrap();

    let store = Store::new(0, init_fn, update_fn);
    let mut lp = Loop::new(store, effecthandler, eventsource, threadpool);

    lp.dispatch(Event::Increment);
    lp.dispatch(Event::Decrement);
    lp.dispatch(Event::Decrement);
    lp.dispatch(Event::Decrement);
}
