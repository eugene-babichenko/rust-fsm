use rust_fsm::*;

state_machine! {
    derive(Debug)
    repr_c(true)
    door(Open)

    Open(Key) => Closed,
    Closed(Key) => Open,
    Open(Break) => Broken,
    Closed(Break) => Broken,
}

#[test]
fn simple() {
    let mut machine = door::StateMachine::new();
    machine.consume(&door::Input::Key).unwrap();
    println!("{:?}", machine.state());
    machine.consume(&door::Input::Key).unwrap();
    println!("{:?}", machine.state());
    machine.consume(&door::Input::Break).unwrap();
    println!("{:?}", machine.state());
}
