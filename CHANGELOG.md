# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog][keepachangelog], and this project
adheres to [Semantic Versioning][semver].

## [Unreleased]
### Changed
* Re-organize repository to import a single crate instead of two.

## [0.2.0] - 2019-05-16
### Added
* Allow setting visibitility for generated state machine types by putting a
  visibility modifier before the state machine name.
* Allow to specify multiple transitions from the same state in a more compact
  form. See the example for the details.
### Changed
* More clear naming:
  * Renamed the `StateMachineWrapper` structure to `StateMachine`;
  * Renamed the `StateMachine` trait to `StateMachineImpl`.
### Removed
* Removed the `consume_anyway` function.

## [0.1.0] - 2019-04-29
### Added
* The `StateMachine` trait for formal definitions of state machines and
  transducers.
* The `StateMachineWrapper` struct - a convenience wrapper around `StateMachine`
  for practical usage.
* The first implementation of the DSL for defining state machines that allows
  to:
  * Define a state machine with its initial state;
  * Define state transitions;
  * Define outputs;
  * Generate boilerplates implementing `StateMachine`.

[keepachangelog]: https://keepachangelog.com/en/1.0.0/
[semver]: https://semver.org/spec/v2.0.0.html

[Unreleased]: https://github.com/eugene-babichenko/rust-fsm/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/eugene-babichenko/rust-fsm/compare/v0.1.0...0.2.0
[0.1.0]: https://github.com/eugene-babichenko/rust-fsm/releases/tag/v0.1.0
