use rust_fsm::state_machine;

state_machine! {
    /// A dummy implementation of the Circuit Breaker pattern to demonstrate
    /// capabilities of its library DSL for defining finite state machines.
    /// https://martinfowler.com/bliki/CircuitBreaker.html
    pub circuit_breaker(Closed)

    Closed(Unsuccessful) => Open [SetupTimer],
    Open(TimerTriggered) => HalfOpen,
    HalfOpen => {
        Successful => Closed,
        Unsuccessful => Open [SetupTimer]
    }
}
