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

mod machine;
mod machine_wrapper;

pub use machine::StateMachine;
pub use machine_wrapper::StateMachineWrapper;
