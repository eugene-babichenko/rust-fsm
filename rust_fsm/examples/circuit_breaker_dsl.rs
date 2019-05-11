/// A dummy implementation of the Circuit Breaker pattern to demonstrate
/// capabilities of its library DSL for defining finite state machines.
/// https://martinfowler.com/bliki/CircuitBreaker.html
#[macro_use]
extern crate rust_fsm_dsl;

use rust_fsm::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;

state_machine! {
    CircuitBreaker(Closed)

    Closed(Unsuccessful) => Open [SetupTimer],
    Open(TimerTriggered) => HalfOpen,
    HalfOpen => {
        Successful => Closed,
        Unsuccessful => Open [SetupTimer]
    }
}

fn main() {
    let machine: StateMachine<CircuitBreaker> = StateMachine::new();

    // Unsuccessful request
    let machine = Arc::new(Mutex::new(machine));
    {
        let mut lock = machine.lock().unwrap();
        let res = lock.consume(&CircuitBreakerInput::Unsuccessful).unwrap();
        assert_eq!(res, Some(CircuitBreakerOutput::SetupTimer));
        assert_eq!(lock.state(), &CircuitBreakerState::Open);
    }

    // Set up a timer
    let machine_wait = machine.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::new(5, 0));
        let mut lock = machine_wait.lock().unwrap();
        let res = lock.consume(&CircuitBreakerInput::TimerTriggered).unwrap();
        assert_eq!(res, None);
        assert_eq!(lock.state(), &CircuitBreakerState::HalfOpen);
    });

    // Try to pass a request when the circuit breaker is still open
    let machine_try = machine.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::new(1, 0));
        let mut lock = machine_try.lock().unwrap();
        let res = lock.consume(&CircuitBreakerInput::Successful);
        assert_eq!(res, Err(()));
        assert_eq!(lock.state(), &CircuitBreakerState::Open);
    });

    // Test if the circit breaker was actually closed
    std::thread::sleep(Duration::new(7, 0));
    {
        let mut lock = machine.lock().unwrap();
        let res = lock.consume(&CircuitBreakerInput::Successful).unwrap();
        assert_eq!(res, None);
        assert_eq!(lock.state(), &CircuitBreakerState::Closed);
    }
}
