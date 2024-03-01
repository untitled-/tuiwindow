use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Field, Fields, Ident, Type};

#[proc_macro_attribute]
pub fn alert_producer(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);

    // Define the new field to be appended
    let new_field = Field {
        attrs: Vec::new(),
        vis: syn::Visibility::Inherited,
        ident: Some(Ident::new(
            "additional_field",
            proc_macro2::Span::call_site(),
        )),
        colon_token: Some(Default::default()),
        ty: Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path::from(Ident::new("String", proc_macro2::Span::call_site())),
        }),
    };

    // Append the new field based on whether the struct has named fields or no fields
    match &mut input.data {
        syn::Data::Struct(data_struct) => {
            match &mut data_struct.fields {
                Fields::Named(fields_named) => {
                    // Struct with named fields
                    fields_named.named.push(new_field);
                }
                Fields::Unit => {
                    // Struct with no fields, convert it to named fields
                    data_struct.fields = Fields::Named(syn::FieldsNamed {
                        brace_token: Default::default(),
                        named: std::iter::once(new_field).collect(),
                    });
                }
                _ => panic!("append_field macro does not support tuple structs."),
            }
        }
        _ => panic!("append_field macro only supports structs."),
    }

    // Generate the updated struct
    let output = quote! {
        #input
    };

    TokenStream::from(output)
}
