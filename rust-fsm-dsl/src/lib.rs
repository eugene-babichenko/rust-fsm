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

    let transitions = input.transitions.iter().flat_map(|def| {
        def.transitions.iter().map(move |transition| Transition {
            initial_state: &def.initial_state,
            input_value: &transition.input_value,
            final_state: &transition.final_state,
            output: &transition.output,
        })
    });

    let mut states = BTreeSet::new();
    let mut inputs = BTreeSet::new();
    let mut outputs = BTreeSet::new();
    let mut transition_cases = Vec::new();
    let mut output_cases = Vec::new();

    #[cfg(feature = "diagram")]
    let mut mermaid_diagram = format!(
        "///```mermaid\n///stateDiagram-v2\n///    [*] --> {}\n",
        input.initial_state
    );

    states.insert(&input.initial_state);

    for transition in transitions {
        let Transition {
            initial_state,
            final_state,
            input_value,
            output,
        } = transition;

        #[cfg(feature = "diagram")]
        mermaid_diagram.push_str(&format!(
            "///    {initial_state} --> {final_state}: {input_value}"
        ));

        transition_cases.push(quote! {
            (Self::State::#initial_state, Self::Input::#input_value) => {
                Some(Self::State::#final_state)
            }
        });

        if let Some(output_value) = output {
            output_cases.push(quote! {
                (Self::State::#initial_state, Self::Input::#input_value) => {
                    Some(Self::Output::#output_value)
                }
            });

            #[cfg(feature = "diagram")]
            mermaid_diagram.push_str(&format!(" [{output_value}]"));
        }

        #[cfg(feature = "diagram")]
        mermaid_diagram.push('\n');

        states.insert(initial_state);
        states.insert(final_state);
        inputs.insert(input_value);
        if let Some(ref output) = output {
            outputs.insert(output);
        }
    }

    #[cfg(feature = "diagram")]
    mermaid_diagram.push_str("///```");
    #[cfg(feature = "diagram")]
    let mermaid_diagram: proc_macro2::TokenStream = mermaid_diagram.parse().unwrap();

    let initial_state_name = &input.initial_state;

    let (input_type, input_impl) = match input.input_type {
        Some(t) => (quote!(#t), quote!()),
        None => (
            quote!(Input),
            quote! {
                #attrs
                pub enum Input {
                    #(#inputs),*
                }
            },
        ),
    };

    let (state_type, state_impl) = match input.state_type {
        Some(t) => (quote!(#t), quote!()),
        None => (
            quote!(State),
            quote! {
                #attrs
                pub enum State {
                    #(#states),*
                }
            },
        ),
    };

    let (output_type, output_impl) = match input.output_type {
        Some(t) => (quote!(#t), quote!()),
        None => {
            // Many attrs and derives may work incorrectly (or simply not work) for empty enums, so we just skip them
            // altogether if the output alphabet is empty.
            let attrs = if outputs.is_empty() {
                quote!()
            } else {
                attrs.clone()
            };
            (
                quote!(Output),
                quote! {
                    #attrs
                    pub enum Output {
                        #(#outputs),*
                    }
                },
            )
        }
    };

    #[cfg(feature = "diagram")]
    let diagram = quote! {
        #[cfg_attr(doc, ::rust_fsm::aquamarine)]
        #mermaid_diagram
    };

    #[cfg(not(feature = "diagram"))]
    let diagram = quote!();

    let output = quote! {
        #diagram
        #visibility mod #fsm_name {
            #attrs
            pub struct Impl;

            pub type StateMachine = ::rust_fsm::StateMachine<Impl>;

            #input_impl
            #state_impl
            #output_impl

            impl ::rust_fsm::StateMachineImpl for Impl {
                type Input = #input_type;
                type State = #state_type;
                type Output = #output_type;
                const INITIAL_STATE: Self::State = Self::State::#initial_state_name;

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
