use std::{cell::RefCell, marker::PhantomData};

use super::{error::StateMachineError, BasicStateMachine, StateWrapper};

pub trait StateMachineBuilder<State, Input, Transition>
where
    Transition: Fn(&State, Input) -> State,
    State: Clone,
{
    type Output;

    /// Starts the builder.
    fn start() -> Self;

    /// Sets particular initial state to the state machine.
    fn initial_state(self, state: State) -> Self;

    /// Sets particular state to the current state.
    fn current_state(self, state: State) -> Self;

    /// Sets particular transition algorithm to the state machine.
    fn transition(self, next: Transition) -> Self;

    /// To finish the builder. If it fails, returns [`crate::machine::error::StateMachineError`].
    fn build(self) -> Result<Self::Output, Box<dyn std::error::Error>>;
}

/// This builder enables us to assemble StateMachine
/// (like [`crate::machine::BasicStateMachine`]) more easily.
pub struct BasicStateMachineBuilder<State, Input, Transition>
where
    Transition: Fn(&State, Input) -> State,
    State: Clone,
{
    initial_state: Option<State>,
    current_state: Option<State>,
    transition: Option<Transition>,
    _marker: PhantomData<Input>,
}

impl<State, Input, Transition> StateMachineBuilder<State, Input, Transition>
    for BasicStateMachineBuilder<State, Input, Transition>
where
    Transition: Fn(&State, Input) -> State,
    State: Clone,
{
    type Output = BasicStateMachine<State, Input, Transition>;

    fn start() -> Self {
        Self::default()
    }

    fn initial_state(mut self, state: State) -> Self {
        self.initial_state = Some(state);
        self
    }

    fn current_state(mut self, state: State) -> Self {
        self.current_state = Some(state);
        self
    }

    fn transition(mut self, next: Transition) -> Self {
        self.transition = Some(next);
        self
    }

    fn build(self) -> Result<Self::Output, Box<dyn std::error::Error>> {
        match (self.initial_state, self.transition) {
            (Some(initial_state), Some(transition)) => Ok(BasicStateMachine {
                initial_state: initial_state.clone(),
                current_state: {
                    // If `current_state` in this builder is still `None`,
                    // sets `initial_state` as the current state forcibly.
                    let current_state = self.current_state;
                    match current_state {
                        Some(state) => RefCell::new(StateWrapper::new(state)),
                        None => RefCell::new(StateWrapper::new(initial_state)),
                    }
                },
                transition,
                _maker: self._marker,
            }),
            (None, _) => Err(Box::new(StateMachineError::MissingField(
                "initial_state".to_string(),
            ))),
            (_, None) => Err(Box::new(StateMachineError::MissingField(
                "transition".to_string(),
            ))),
        }
    }
}

impl<State, Input, Transition> Default for BasicStateMachineBuilder<State, Input, Transition>
where
    Transition: Fn(&State, Input) -> State,
    State: Clone,
{
    fn default() -> Self {
        BasicStateMachineBuilder {
            initial_state: None,
            current_state: None,
            transition: None,
            _marker: PhantomData::<Input>::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{BasicStateMachineBuilder, StateMachineBuilder};
    use crate::machine::StateMachine;

    #[allow(dead_code)]
    #[derive(Copy, Clone, Debug, PartialEq)]
    enum Stations {
        Shibuya,
        IkejiriOhashi,
        Sangendyaya,
        KomazawaDaigaku,
        Sakurashinmachi,
        Yoga,
        FutakoTamagawa,
    }

    #[allow(dead_code)]
    enum Train {
        Local,
        Express,
    }

    #[test]
    fn test_build() {
        // sets only initial state
        let sm = BasicStateMachineBuilder::start()
            .initial_state(Stations::Shibuya)
            .transition(|station, train| match (station, train) {
                (Stations::Shibuya, Train::Local) => Stations::IkejiriOhashi,
                (Stations::Shibuya, Train::Express) => Stations::Sangendyaya,
                (Stations::IkejiriOhashi, Train::Local) => Stations::Sangendyaya,
                (Stations::Sangendyaya, Train::Local) => Stations::KomazawaDaigaku,
                (Stations::Sangendyaya, Train::Express) => Stations::FutakoTamagawa,
                (Stations::KomazawaDaigaku, Train::Local) => Stations::Sakurashinmachi,
                (Stations::Sakurashinmachi, Train::Local) => Stations::Yoga,
                _ => unreachable!(),
            })
            .build()
            .unwrap();

        assert_eq!(Stations::Shibuya, sm.current_state());

        // sets current state after initializing initial state
        let sm = BasicStateMachineBuilder::start()
            .initial_state(Stations::Shibuya)
            .current_state(Stations::Sangendyaya)
            .transition(|station, train| match (station, train) {
                (Stations::Shibuya, Train::Local) => Stations::IkejiriOhashi,
                (Stations::Shibuya, Train::Express) => Stations::Sangendyaya,
                (Stations::IkejiriOhashi, Train::Local) => Stations::Sangendyaya,
                (Stations::Sangendyaya, Train::Local) => Stations::KomazawaDaigaku,
                (Stations::Sangendyaya, Train::Express) => Stations::FutakoTamagawa,
                (Stations::KomazawaDaigaku, Train::Local) => Stations::Sakurashinmachi,
                (Stations::Sakurashinmachi, Train::Local) => Stations::Yoga,
                _ => unreachable!(),
            })
            .build()
            .unwrap();

        assert_eq!(Stations::Sangendyaya, sm.current_state());
    }

    #[test]
    fn test_fail_initial_state() {
        let sm = BasicStateMachineBuilder::start()
            .transition(|station, train| match (station, train) {
                (Stations::Shibuya, Train::Local) => Stations::IkejiriOhashi,
                (Stations::Shibuya, Train::Express) => Stations::Sangendyaya,
                (Stations::IkejiriOhashi, Train::Local) => Stations::Sangendyaya,
                (Stations::Sangendyaya, Train::Local) => Stations::KomazawaDaigaku,
                (Stations::Sangendyaya, Train::Express) => Stations::FutakoTamagawa,
                (Stations::KomazawaDaigaku, Train::Local) => Stations::Sakurashinmachi,
                (Stations::Sakurashinmachi, Train::Local) => Stations::Yoga,
                _ => unreachable!(),
            })
            .build();

        assert!(sm.is_err());
    }
}
