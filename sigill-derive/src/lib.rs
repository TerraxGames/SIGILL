use proc_macro::TokenStream;
use quote::quote;
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

    'f: for field in fields.iter() {
        for attr in field.attrs.iter() {
            if attr.path().is_ident("deref") {
                field_type = Some(field.ty.clone());
                break 'f;
            }
        }
    }

    if field_type.is_none() {
        if let Some(field) = fields.iter().nth(0) {
            field_type = Some(field.ty.clone());
        } else {
            panic!("No default field or field with #[deref] attribute found.");
        }
    }
    
    let output = {
        quote! {
            impl #generics std::ops::Deref for #ident #generics {
                type Target = #field_type;

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }
        }
    };

    output.into()
}
