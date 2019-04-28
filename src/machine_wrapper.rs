use crate::StateMachine;

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
