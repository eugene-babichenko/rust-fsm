#[macro_use]
extern crate rust_fsm_dsl;

use rust_fsm::*;

state_machine! {
    Door(Open)

    Open(Key) => Closed,
    Closed(Key) => Open,
    Open(Break) => Broken,
    Closed(Break) => Broken,
}

fn main() {
    let mut machine: StateMachine<Door> = StateMachine::new();
    machine.consume(&DoorInput::Key).unwrap();
    println!("{:?}", machine.state());
    machine.consume(&DoorInput::Key).unwrap();
    println!("{:?}", machine.state());
    machine.consume(&DoorInput::Break).unwrap();
    println!("{:?}", machine.state());
}
