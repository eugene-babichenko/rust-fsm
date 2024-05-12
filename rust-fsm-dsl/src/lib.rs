//! DSL implementation for defining finite state machines for `rust-fsm`. See
//! more in the `rust-fsm` crate documentation.

#![recursion_limit = "128"]
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use std::{collections::BTreeSet, iter::FromIterator};
use syn::{parse_macro_input, Ident};

mod parser;

/// The full information about a state transition. Used to unify the
/// represantion of the simple and the compact forms.
struct Transition<'a> {
    initial_state: &'a Ident,
    input_value: &'a Ident,
    final_state: &'a Ident,
    output: &'a Option<Ident>,
}

#[proc_macro]
/// Produce a state machine definition from the provided `rust-fmt` DSL
/// description.
pub fn state_machine(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as parser::StateMachineDef);

    let attrs = input
        .attributes
        .into_iter()
        .map(ToTokens::into_token_stream);
    let attrs = proc_macro2::TokenStream::from_iter(attrs);

    if input.transitions.is_empty() {
        let output = quote! {
            compile_error!("rust-fsm: at least one state transition must be provided");
        };
        return output.into();
    }

    let fsm_name = input.name;
    let visibility = input.visibility;

    let transitions: Vec<_> = input
        .transitions
        .iter()
        .flat_map(|def| {
            def.transitions.iter().map(move |transition| Transition {
                initial_state: &def.initial_state,
                input_value: &transition.input_value,
                final_state: &transition.final_state,
                output: &transition.output,
            })
        })
        .collect();

    let mut states = BTreeSet::new();
    let mut inputs = BTreeSet::new();
    let mut outputs = BTreeSet::new();

    states.insert(&input.initial_state);

    for transition in transitions.iter() {
        states.insert(transition.initial_state);
        states.insert(transition.final_state);
        inputs.insert(transition.input_value);
        if let Some(ref output) = transition.output {
            outputs.insert(output);
        }
    }

    let initial_state_name = &input.initial_state;

    let mut transition_cases = vec![];
    for transition in transitions.iter() {
        let initial_state = &transition.initial_state;
        let input_value = &transition.input_value;
        let final_state = &transition.final_state;
        transition_cases.push(quote! {
            (State::#initial_state, Input::#input_value) => {
                Some(State::#final_state)
            }
        });
    }

    let mut output_cases = vec![];
    for transition in transitions.iter() {
        if let Some(output_value) = &transition.output {
            let initial_state = &transition.initial_state;
            let input_value = &transition.input_value;
            output_cases.push(quote! {
                (State::#initial_state, Input::#input_value) => {
                    Some(Output::#output_value)
                }
            });
        }
    }

    // Many attrs and derives may work incorrectly (or simply not work) for
    // empty enums, so we just skip them altogether if the output alphabet is
    // empty.
    let output_attrs = if outputs.is_empty() {
        quote!()
    } else {
        attrs.clone()
    };

    let output = quote! {
        #visibility mod #fsm_name {
            #attrs
            pub struct Impl;

            pub type StateMachine = rust_fsm::StateMachine<Impl>;

            #attrs
            pub enum Input {
                #(#inputs),*
            }

            #attrs
            pub enum State {
                #(#states),*
            }

            #output_attrs
            pub enum Output {
                #(#outputs),*
            }

            impl rust_fsm::StateMachineImpl for Impl {
                type Input = Input;
                type State = State;
                type Output = Output;
                const INITIAL_STATE: Self::State = State::#initial_state_name;

                fn transition(state: &Self::State, input: &Self::Input) -> Option<Self::State> {
                    match (state, input) {
                        #(#transition_cases)*
                        _ => None,
                    }
                }

                fn output(state: &Self::State, input: &Self::Input) -> Option<Self::Output> {
                    match (state, input) {
                        #(#output_cases)*
                        _ => None,
                    }
                }
            }
        }
    };

    output.into()
}
