//! A framework for building finite state machines in Rust
//!
//! The `rust-fsm` crate provides a simple and universal framework for building
//! state machines in Rust with minimum effort. The essential part of this crate
//! is the [`StateMachine`] trait. This trait allows a developer to provide a
//! strict state machine definition, e.g. specify its:
//!
//! * An input alphabet - a set of entities that the state machine takes as
//!   inputs and performs state transitions based on them.
//! * Possible states - a set of states this machine could be in.
//! * An output alphabet - a set of entities that the state machine may output
//!   as results of its work.
//! * A transition function - a function that changes the state of the state
//!   machine based on its current state and the provided input.
//! * An output function - a function that outputs something from the output
//!   alphabet based on the current state and the provided inputs.
//! * The initial state of the machine.
//!
//! Note that on the implementation level such abstraction allows build any type
//! of state machines:
//!
//! * A classical state machine by providing only an input alphabet, a set of
//!   states and a transition function.
//! * A Mealy machine by providing all entities listed above.
//! * A Moore machine by providing an output function that do not depend on the
//!   provided inputs.
//!
//! # Use
//!
//! All you need to do to build a state machine is to implement the
//! `StateMachine` trait and use it in conjuctions with some of the provided
//! wrappers (for now there is only `StateMachineWrapper`).
//!
//! You can see an example of the Circuit Breaker state machine in the
//! [project repository][repo].
//!
//! [repo]: https://github.com/eugene-babichenko/rust-fsm/blob/master/examples/circuit_breaker.rs

/// This trait is designed to describe any possible deterministic finite state
/// machine/transducer. This is just a formal definition that may be
/// inconvenient to be used in practical programming, but it is used throughout
/// this library for more practical things.
pub trait StateMachine {
    /// The input alphabet.
    type Input;
    /// The set of possible states.
    type State: Copy;
    /// The output alphabet.
    type Output;
    /// The initial state of the machine.
    const INITIAL_STATE: Self::State;
    /// The transition fuction that outputs a new state based on the current
    /// state and the provided input. Outputs `None` when there is no transition
    /// for a given combination of the input and the state.
    fn transition(state: &Self::State, input: &Self::Input) -> Option<Self::State>;
    /// The output function that outputs some value from the output alphabet
    /// based on the current state and the given input. Outputs `None` when
    /// there is no output for a given combination of the input and the state.
    fn output(state: &Self::State, input: &Self::Input) -> Option<Self::Output>;
}

/// A convenience wrapper around the `StateMachine` trait that encapsulates the
/// state and transition and output function calls.
pub struct StateMachineWrapper<T: StateMachine> {
    state: T::State,
}

impl<T> StateMachineWrapper<T>
where
    T: StateMachine,
{
    /// Create a new instance of this wrapper which encapsulates the initial
    /// state.
    pub fn new() -> Self {
        StateMachineWrapper {
            state: T::INITIAL_STATE,
        }
    }

    /// Consumes the provided input, gives an output and performs a state
    /// transition. If a state transition with the current state and the
    /// provided input is not allowed, returns an error.
    pub fn consume(&mut self, input: &T::Input) -> Result<Option<T::Output>, ()> {
        // Operations are reodered for optimization. When the transition is not
        // allowed this code exits as soon as possible without calculating the
        // output.
        let state = match T::transition(&self.state, input) {
            Some(state) => state,
            None => return Err(()),
        };
        let output = T::output(&self.state, input);
        self.state = state;
        Ok(output)
    }

    /// Consumes the provided input, gives an output and performs a state
    /// transition. If a state transition is not allowed, this function just
    /// provides an output.
    pub fn consume_anyway(&mut self, input: &T::Input) -> Option<T::Output> {
        let output = T::output(&self.state, input);
        if let Some(state) = T::transition(&self.state, input) {
            self.state = state;
        }
        output
    }

    /// Returns the current state.
    pub fn state(&self) -> &T::State {
        &self.state
    }
}
