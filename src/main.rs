use mobius::*;

type Model = u64;

#[derive(Debug, Clone)]
enum Event {
    Increment,
    Decrement,
}

#[derive(Debug, Clone)]
enum Effect {
    PrintOut(&'static str),
    PrintError(&'static str),
}

struct Logic;

impl Logic {
    fn init(model: &Model) -> First<Model, Effect> {
        dbg!(model);
        First::from(1)
    }

    fn update(model: &Model, event: Event) -> Next<Model, Effect> {
        match event {
            Event::Increment => Next::from(model + 1),
            Event::Decrement => {
                if *model == 0 {
                    Next::dispatch(Effect::PrintError("model cannot go negative"))
                } else {
                    Next::from(model - 1)
                }
            }
        }
    }

    fn handle(__: &Model, effect: Effect) -> Vec<Event> {
        match effect {
            Effect::PrintOut(s) => println!("{}", s),
            Effect::PrintError(s) => eprintln!("{}", s),
        }
        vec![]
    }

    fn observe(model: &Model) {
        println!("observe: {:#?}", model);
    }
}

fn main() {
    let loupe = LoopBuilder::new(Logic::update, Logic::handle)
        .observe(Logic::observe)
        .start(0);

    loupe.dispatch(Event::Increment).dispatch(Event::Increment);
    loupe
        .dispatch(Event::Decrement)
        .dispatch(Event::Decrement)
        .dispatch(Event::Decrement);

    loupe.run();
}
