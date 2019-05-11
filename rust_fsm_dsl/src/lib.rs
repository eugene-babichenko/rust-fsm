//! DSL implementation for defining finite state machines for `rust-fsm`. See
//! more in the `rust-fsm` crate documentation.

#![recursion_limit = "128"]
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::{
    braced, bracketed, parenthesized,
    parse::{Error, Parse, ParseStream, Result},
    parse_macro_input,
    token::{Bracket, Paren},
    Ident, Token, Visibility,
};

struct Output(Option<Ident>);

impl Parse for Output {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.lookahead1().peek(Bracket) {
            let output_content;
            bracketed!(output_content in input);
            Ok(Self(Some(output_content.parse()?)))
        } else {
            Ok(Self(None))
        }
    }
}

impl Into<Option<Ident>> for Output {
    fn into(self) -> Option<Ident> {
        self.0
    }
}

struct TransitionEntry {
    input_value: Ident,
    final_state: Ident,
    output: Option<Ident>,
}

impl Parse for TransitionEntry {
    fn parse(input: ParseStream) -> Result<Self> {
        let input_value = input.parse()?;
        input.parse::<Token![=>]>()?;
        let final_state = input.parse()?;
        let output = input.parse::<Output>()?.into();
        Ok(Self {
            input_value,
            final_state,
            output,
        })
    }
}

struct TransitionDef {
    initial_state: Ident,
    transitions: Vec<TransitionEntry>,
}

impl Parse for TransitionDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let initial_state = input.parse()?;
        let transitions = if input.lookahead1().peek(Paren) {
            let input_content;
            parenthesized!(input_content in input);
            let input_value = input_content.parse()?;
            input.parse::<Token![=>]>()?;
            let final_state = input.parse()?;
            let output = input.parse::<Output>()?.into();

            vec![TransitionEntry {
                input_value,
                final_state,
                output,
            }]
        } else {
            input.parse::<Token![=>]>()?;
            let entries_content;
            braced!(entries_content in input);

            let entries: Vec<_> = entries_content
                .parse_terminated::<_, Token![,]>(TransitionEntry::parse)?
                .into_iter()
                .collect();
            if entries.is_empty() {
                return Err(Error::new_spanned(
                    initial_state,
                    "No transitions provided for a compact representation",
                ));
            }
            entries
        };
        Ok(Self {
            initial_state,
            transitions,
        })
    }
}

struct StateMachineDef {
    visibility: Visibility,
    name: Ident,
    initial_state: Ident,
    transitions: Vec<TransitionDef>,
}

impl Parse for StateMachineDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let visibility = input.parse()?;
        let name = input.parse()?;

        let initial_state_content;
        parenthesized!(initial_state_content in input);
        let initial_state = initial_state_content.parse()?;

        let transitions = input
            .parse_terminated::<_, Token![,]>(TransitionDef::parse)?
            .into_iter()
            .collect();

        Ok(Self {
            visibility,
            name,
            initial_state,
            transitions,
        })
    }
}

struct Transition<'a> {
    initial_state: &'a Ident,
    input_value: &'a Ident,
    final_state: &'a Ident,
    output: &'a Option<Ident>,
}

#[proc_macro]
pub fn state_machine(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as StateMachineDef);

    if input.transitions.is_empty() {
        let output = quote! {
            compile_error!("rust-fsm: at least one state transition must be provided");
        };
        return output.into();
    }

    let struct_name = input.name;
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

    let mut states = HashSet::new();
    let mut inputs = HashSet::new();
    let mut outputs = HashSet::new();

    states.insert(&input.initial_state);

    for transition in transitions.iter() {
        states.insert(&transition.initial_state);
        states.insert(&transition.final_state);
        inputs.insert(&transition.input_value);
        if let Some(ref output) = transition.output {
            outputs.insert(output);
        }
    }

    let states_enum_name = Ident::new(&format!("{}State", struct_name), struct_name.span());
    let initial_state_name = &input.initial_state;

    let inputs_enum_name = Ident::new(&format!("{}Input", struct_name), struct_name.span());

    let mut transition_cases = vec![];
    for transition in transitions.iter() {
        let initial_state = &transition.initial_state;
        let input_value = &transition.input_value;
        let final_state = &transition.final_state;
        transition_cases.push(quote! {
            (#states_enum_name::#initial_state, #inputs_enum_name::#input_value) => {
                Some(#states_enum_name::#final_state)
            }
        });
    }

    let (outputs_repr, outputs_type, output_impl) = if !outputs.is_empty() {
        let outputs_type_name = Ident::new(&format!("{}Output", struct_name), struct_name.span());
        let outputs_repr = quote! {
            #[derive(Debug, PartialEq)]
            #visibility enum #outputs_type_name {
                #(#outputs),*
            }
        };

        let outputs_type = quote! { #outputs_type_name };

        let mut output_cases = vec![];
        for transition in transitions.iter() {
            if let Some(output_value) = &transition.output {
                let initial_state = &transition.initial_state;
                let input_value = &transition.input_value;
                output_cases.push(quote! {
                    (#states_enum_name::#initial_state, #inputs_enum_name::#input_value) => {
                        Some(#outputs_type_name::#output_value)
                    }
                });
            }
        }

        let output_impl = quote! {
            match (state, input) {
                #(#output_cases)*
                _ => None,
            }
        };

        (outputs_repr, outputs_type, output_impl)
    } else {
        (quote! {}, quote! { () }, quote! {None})
    };

    let output = quote! {
        #visibility struct #struct_name;

        #[derive(Clone, Copy, Debug, PartialEq)]
        #visibility enum #states_enum_name {
            #(#states),*
        }

        #[derive(Debug, PartialEq)]
        #visibility enum #inputs_enum_name {
            #(#inputs),*
        }

        #outputs_repr

        impl rust_fsm::StateMachineImpl for #struct_name {
            type Input = #inputs_enum_name;
            type State = #states_enum_name;
            type Output = #outputs_type;
            const INITIAL_STATE: Self::State = #states_enum_name::#initial_state_name;

            fn transition(state: &Self::State, input: &Self::Input) -> Option<Self::State> {
                match (state, input) {
                    #(#transition_cases)*
                    _ => None,
                }
            }

            fn output(state: &Self::State, input: &Self::Input) -> Option<Self::Output> {
                #output_impl
            }
        }
    };

    output.into()
}
