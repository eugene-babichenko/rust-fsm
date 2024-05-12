#![doc = include_str!("../../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

use core::fmt;
#[cfg(feature = "std")]
use std::error::Error;

#[cfg(feature = "dsl")]
pub use rust_fsm_dsl::state_machine;

#[cfg(feature = "diagram")]
pub use aquamarine::aquamarine;

/// This trait is designed to describe any possible deterministic finite state
/// machine/transducer. This is just a formal definition that may be
/// inconvenient to be used in practical programming, but it is used throughout
/// this library for more practical things.
pub trait StateMachineImpl {
    /// The input alphabet.
    type Input;
    /// The set of possible states.
    type State;
    /// The output alphabet.
    type Output;
    /// The initial state of the machine.
    // allow since there is usually no interior mutability because states are enums
    #[allow(clippy::declare_interior_mutable_const)]
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
#[derive(Debug, Clone)]
pub struct StateMachine<T: StateMachineImpl> {
    state: T::State,
}

#[derive(Debug, Clone)]
/// An error type that represents that the state transition is impossible given
/// the current combination of state and input.
pub struct TransitionImpossibleError;

impl<T> StateMachine<T>
where
    T: StateMachineImpl,
{
    /// Create a new instance of this wrapper which encapsulates the initial
    /// state.
    pub fn new() -> Self {
        Self::from_state(T::INITIAL_STATE)
    }

    /// Create a new instance of this wrapper which encapsulates the given
    /// state.
    pub fn from_state(state: T::State) -> Self {
        Self { state }
    }

    /// Consumes the provided input, gives an output and performs a state
    /// transition. If a state transition with the current state and the
    /// provided input is not allowed, returns an error.
    pub fn consume(
        &mut self,
        input: &T::Input,
    ) -> Result<Option<T::Output>, TransitionImpossibleError> {
        if let Some(state) = T::transition(&self.state, input) {
            let output = T::output(&self.state, input);
            self.state = state;
            Ok(output)
        } else {
            Err(TransitionImpossibleError)
        }
    }

    /// Returns the current state.
    pub fn state(&self) -> &T::State {
        &self.state
    }
}

impl<T> Default for StateMachine<T>
where
    T: StateMachineImpl,
{
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TransitionImpossibleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "cannot perform a state transition from the current state with the provided input"
        )
    }
}

#[cfg(feature = "std")]
impl Error for TransitionImpossibleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
