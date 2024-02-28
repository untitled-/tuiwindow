extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Lit, Meta, MetaNameValue};

#[proc_macro_derive(Greet, attributes(greeting))]
pub fn derive_greet(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident; // Gets the struct name

    // Initialize a variable to hold an optional custom greeting message
    let mut custom_greeting = None;

    // Look for attributes named `greeting` and extract their values
    for attr in input.attrs {
        if let Ok(Meta::NameValue(MetaNameValue {
            path,
            lit: Lit::Str(lit_str),
            ..
        })) = attr.parse_meta()
        {
            if path.is_ident("greeting") {
                custom_greeting = Some(lit_str.value());
                break;
            }
        }
    }
    let greeting = match custom_greeting {
        Some(greeting) => quote! { #greeting },
        None => quote! { format!("Hello, I am a {}", stringify!(#name)) },
    };
    // Generate the trait implementation
    let expanded = quote! {
        impl Greet for #name {
            fn greet(&self) -> String {
                #greeting
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(RenderComponent, attributes(focusable))]
pub fn derive_render_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident; // Gets the struct name

    // Initialize a variable to hold an optional custom greeting message
    let mut custom_greeting = None;

    // Look for attributes named `greeting` and extract their values
    for attr in input.attrs {
        if let Ok(Meta::NameValue(MetaNameValue {
            path,
            lit: Lit::Str(lit_str),
            ..
        })) = attr.parse_meta()
        {
            dbg!(lit_str.value());
            if path.is_ident("focusable") {
                custom_greeting = Some(lit_str.value());
                break;
            }
        }
    }
    let body = match custom_greeting {
        Some(_) => quote! { Component::new_focusable(value) },
        None => quote! { Component::new(value) },
    };
    // Generate the trait implementation
    let expanded = quote! {
        impl From<#name> for Component {
            fn from(value: #name) -> Self {
              #body
            }

        }

        impl AsAny for #name {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    };

    TokenStream::from(expanded)
}
