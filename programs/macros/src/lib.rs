extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataEnum, DeriveInput};

#[proc_macro_derive(ValidateLifecycle)]
pub fn my_auto_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Pattern match to ensure we are dealing with an enum
    let data_enum = match input.data {
        Data::Enum(data_enum) => data_enum,
        _ => panic!("MyAutoDerive is only defined for enums"),
    };

    let name = input.ident; // Get the name of the enum

    // Generate code for each variant
    let variants = data_enum.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let func_call = quote! {
            Self::#variant_name(inner) => inner.some_function(),
        };
        func_call
    });

    let expanded = quote! {
        impl #name {
            pub fn my_function(&self) -> ReturnType {
                match self {
                    #(#variants)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
