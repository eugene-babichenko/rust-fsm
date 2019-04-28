/// This trait is designed to describe any possible deterministic finite state
/// machine/transducer. This is just a formal definition that may be
/// inconvenient to be used in practical programming, but it is used throughout
/// this library for more practical things.
pub trait StateMachine {
    /// The input alphabet.
    type Input;
    /// The set of possible states.
    type State;
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
