extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Field, Ident};

fn data_fields(data: &Data) -> impl Iterator<Item = (&Ident, &Field)> {
    match data {
        Data::Struct(data_struct) => data_struct
            .fields
            .iter()
            .map(|f| (f.ident.as_ref().unwrap(), f)),
        _ => unimplemented!(),
    }
}

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);

    let struct_name = derive_input.ident;
    let builder_name = Ident::new(&format!("{}Builder", struct_name), struct_name.span());

    let builder_struct_fields = data_fields(&derive_input.data).map(|(name, f)| {
        let ty = &f.ty;

        quote! { #name : Option< #ty > }
    });

    let fields = data_fields(&derive_input.data).map(|(name, _)| name);

    let builder_impls = data_fields(&derive_input.data).map(|(name, f)| {
        let ty = &f.ty;

        quote! {
            fn #name (&mut self, #name: #ty) -> &mut Self {
                self.#name = Some( #name );
                self
            }
        }
    });

    let build_fn_impl = data_fields(&derive_input.data).map(|(name, _)| {
        let err = format!("Missing field: {}", name);
        quote! { #name: self.#name.take().ok_or(#err)? }
    });

    TokenStream::from(quote! {
        pub struct #builder_name {
            #(#builder_struct_fields,)*
        }

        impl #builder_name {
            #(#builder_impls)*

            pub fn build(&mut self) -> Result<#struct_name, Box<dyn std::error::Error>> {
                Ok(#struct_name {
                    #(#build_fn_impl,)*
                })
            }
        }

        impl #struct_name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#fields: None,)*
                }
            }
        }
    })
}
