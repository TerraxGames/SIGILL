use std::str::FromStr;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(Deref, attributes(deref))]
pub fn derive_deref(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, generics, data, .. } = parse_macro_input!(input);
    let fields;
    if let Data::Struct(data_struct) = data {
        fields = data_struct.fields;
    } else {
        panic!("Only structs may derive Deref.");
    }

    let mut field_type = None;
    let mut field_name = None;

    'f: for (i, field) in fields.iter().enumerate() {
        for attr in field.attrs.iter() {
            if attr.path().is_ident("deref") {
                field_type = Some(field.ty.clone());
                if let Some(ref ident) = field.ident {
                    field_name = Some(ident.to_token_stream());
                } else {
                    field_name = Some(syn::Index::from(i).to_token_stream());
                }
                break 'f;
            }
        }
    }

    if field_name.is_none() {
        if let Some(field) = fields.iter().nth(0) {
            field_type = Some(field.ty.clone());
            if let Some(ref ident) = field.ident {
                field_name = Some(ident.to_token_stream());
            } else {
                field_name = Some(syn::Index::from(0).to_token_stream());
            }
        } else {
            panic!("No default field or field with #[deref] attribute found.");
        }
    }

    let where_clause = if let Some(ref where_clause) = generics.where_clause {
        where_clause.to_token_stream()
    } else {
        proc_macro2::TokenStream::from_str("").unwrap()
    };
    
    let output = {
        quote! {
            impl #generics std::ops::Deref for #ident #generics #where_clause {
                type Target = #field_type;

                fn deref(&self) -> &Self::Target {
                    &self.#field_name
                }
            }
        }
    };

    output.into()
}

#[proc_macro_derive(DerefMut)]
pub fn derive_deref_mut(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, generics, data, .. } = parse_macro_input!(input);
    let fields;
    if let Data::Struct(data_struct) = data {
        fields = data_struct.fields;
    } else {
        panic!("Only structs may derive Deref.");
    }

    let mut field_name = None;

    'f: for (i, field) in fields.iter().enumerate() {
        for attr in field.attrs.iter() {
            if attr.path().is_ident("deref") {
                if let Some(ref ident) = field.ident {
                    field_name = Some(ident.to_token_stream());
                } else {
                    field_name = Some(syn::Index::from(i).to_token_stream());
                }
                break 'f;
            }
        }
    }

    if field_name.is_none() {
        if let Some(field) = fields.iter().nth(0) {
            if let Some(ref ident) = field.ident {
                field_name = Some(ident.to_token_stream());
            } else {
                field_name = Some(syn::Index::from(0).to_token_stream());
            }
        } else {
            panic!("No default field or field with #[deref] attribute found. Ensure that Deref has been derived first.");
        }
    }

    let where_clause = if let Some(ref where_clause) = generics.where_clause {
        where_clause.to_token_stream()
    } else {
        proc_macro2::TokenStream::from_str("").unwrap()
    };
    
    let output = {
        quote! {
            impl #generics std::ops::DerefMut for #ident #generics #where_clause {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.#field_name
                }
            }
        }
    };

    output.into()
}
