/// A dummy implementation of the Circuit Breaker pattern to demonstrate
/// capabilities of its library DSL for defining finite state machines.
/// https://martinfowler.com/bliki/CircuitBreaker.html
use rust_fsm::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;

state_machine! {
    circuit_breaker(Closed)

    Closed(Unsuccessful) => Open [SetupTimer],
    Open(TimerTriggered) => HalfOpen,
    HalfOpen => {
        Successful => Closed,
        Unsuccessful => Open [SetupTimer]
    }
}

#[test]
fn circit_breaker_dsl() {
    let machine = circuit_breaker::StateMachine::new();

    // Unsuccessful request
    let machine = Arc::new(Mutex::new(machine));
    {
        let mut lock = machine.lock().unwrap();
        let res = lock.consume(&circuit_breaker::Input::Unsuccessful).unwrap();
        assert!(matches!(res, Some(circuit_breaker::Output::SetupTimer)));
        assert!(matches!(lock.state(), &circuit_breaker::State::Open));
    }

    // Set up a timer
    let machine_wait = machine.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::new(5, 0));
        let mut lock = machine_wait.lock().unwrap();
        let res = lock
            .consume(&circuit_breaker::Input::TimerTriggered)
            .unwrap();
        assert!(matches!(res, None));
        assert!(matches!(lock.state(), &circuit_breaker::State::HalfOpen));
    });

    // Try to pass a request when the circuit breaker is still open
    let machine_try = machine.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::new(1, 0));
        let mut lock = machine_try.lock().unwrap();
        let res = lock.consume(&circuit_breaker::Input::Successful);
        assert!(matches!(res, Err(TransitionImpossibleError)));
        assert!(matches!(lock.state(), &circuit_breaker::State::Open));
    });

    // Test if the circit breaker was actually closed
    std::thread::sleep(Duration::new(7, 0));
    {
        let mut lock = machine.lock().unwrap();
        let res = lock.consume(&circuit_breaker::Input::Successful).unwrap();
        assert!(matches!(res, None));
        assert!(matches!(lock.state(), &circuit_breaker::State::Closed));
    }
}
