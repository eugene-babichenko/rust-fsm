//! A framework for building finite state machines in Rust
//!
//! The `rust-fsm` crate provides a simple and universal framework for building
//! state machines in Rust with minimum effort. This is achieved by two
//! components:
//!
//! * The `rust-fsm` crate, that provides data types for building state machines
//!   and convenience wrappers for these types.
//! * The `rust-fsm-dsl` crate, that contains the `state_machine` macro that
//!   parses a simple DSL and generates all boilerplate code for the described
//!   state machine.
//!
//! The essential part of this crate is the [`StateMachine`] trait. This trait
//! allows a developer to provide a strict state machine definition, e.g.
//! specify its:
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
//! Initially this library was designed to build an easy to use DSL for defining
//! state machines on top of it. Using the DSL will require to connect an
//! additional crate `rust-fsm-dsl` (this is due to limitation of the procedural
//! macros system).
//!
//! ## Using the DSL for defining state machines
//!
//! The DSL is parsed by the `state_machine` macro. Here is a little example.
//!
//! ```rust,ignore
//! #[macro_use]
//! extern crate rust_fsm_dsl;
//!
//! use rust_fsm::*;
//!
//! state_machine! {
//!     CircuitBreaker(Closed)
//!
//!     Closed(Unsuccessful) => Open [SetupTimer],
//!     Open(TimerTriggered) => HalfOpen,
//!     HalfOpen(Successful) => Closed,
//!     HalfOpen(Unsuccessful) => Open [SetupTimer],
//! }
//! ```
//!
//! This code sample:
//!
//! * Defines a state machine called `CircuitBreaker`;
//! * Sets the initial state of this state machine to `Closed`;
//! * Defines state transitions. For example: on receiving the `Successful`
//!   input when in the `HalfOpen` state, the machine must move to the `Closed`
//!   state;
//! * Defines outputs. For example: on receiving `Unsuccessful` in the
//!   `Closed` state, the machine must output `SetupTimer`.
//!
//! This state machine can be used as follows:
//!
//! ```rust,ignore
//! // Initialize the state machine. The state is `Closed` now.
//! let mut machine: StateMachineWrapper<CircuitBreaker> = StateMachineWrapper::new();
//! // Consume the `Successful` input. No state transition is performed. Output
//! // is `None`.
//! machine.consume_anyway(&CircuitBreakerInput::Successful);
//! // Consume the `Unsuccesful` input. The machine is moved to the `Open`
//! // state. The output is `SetupTimer`.
//! let output = machine.consume_anyway(&CircuitBreakerInput::Unsuccesful);
//! // Check the output
//! if output == Some(CircuitBreakerOutput::SetupTimer) {
//!     // Set up the timer...
//! }
//! // Check the state
//! if machine.state() == &CircuitBreakerState::Open {
//!     // Do something...
//! }
//! ```
//!
//! As you can see, the following entities are generated:
//!
//! * An empty structure `CircuitBreaker` that implements the `StateMachine`
//!   trait.
//! * Enums `CircuitBreakerState`, `CircuitBreakerInput` and
//!   `CircuitBreakerOutput` that represent the state, the input alphabet and
//!   the output alphabet respectively.
//!
//! Note that if there is no outputs in the specification, the output alphabet
//! is set to `()`. The set of states and the input alphabet must be non-empty
//! sets.
//!
//! ## Without DSL
//!
//! The `state_machine` macro has limited capabilities (for example, a state
//! cannot carry any additional data), so in certain complex cases a user might
//! want to write a more complex state machine by hand.
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

impl<T> Default for StateMachineWrapper<T>
where
    T: StateMachine,
{
    fn default() -> Self {
        Self::new()
    }
}
