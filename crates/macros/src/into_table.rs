use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

use crate::common::sanitize_field_string;

pub fn into_table(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;

    let mut table_fields: Vec<proc_macro2::TokenStream> = vec![];

    if let syn::Data::Struct(data_struct) = &input.data {
        match &data_struct.fields {
            syn::Fields::Named(named_fields) => {
                for field in named_fields.named.iter() {
                    let field_name = &field.ident;
                    let field_str = sanitize_field_string(field_name.as_ref().unwrap().to_string());

                    if let syn::Type::Path(path_type) = &field.ty {
                        for segment in path_type.path.segments.iter() {
                            let field_type = &segment.ident;

                            if field_type == "Option" {
                                table_fields.push(quote! {
                                    if let Some(value) = self.#field_name {
                                        out.set(#field_str, value)?;
                                    }
                                });
                            } else {
                                table_fields.push(quote! {
                                    out.set(#field_str, self.#field_name)?;
                                });
                            }
                        }
                    }
                }
            }
            _ => {
                panic!("IntoTable only works on structs with named fields");
            }
        }
    } else {
        panic!("IntoTable only works on structs");
    }

    let table_fields = proc_macro2::TokenStream::from_iter(table_fields);

    let generics = &input.generics.params;

    let expand;

    let into_lua_body = quote! {
        let out = lua.create_table()?;

        #table_fields

        Ok(mlua::Value::Table(out))
    };

    if generics.is_empty() {
        expand = quote! {
            impl<'a> mlua::IntoLua<'a> for #struct_name {
                fn into_lua(self, lua: &'a Lua) -> mlua::Result<mlua::Value<'a>> {
                    #into_lua_body
                }
            }
        };
    } else {
        let lt;
        let param = generics.first().unwrap();

        if let syn::GenericParam::Lifetime(ltp) = param {
            lt = ltp.clone();
        } else {
            lt = syn::LifetimeParam::new(syn::Lifetime::new("'lua", Span::call_site()));
        }

        expand = quote! {
            impl<#generics> mlua::IntoLua<#lt> for #struct_name<#generics> {
                fn into_lua(self, lua: &#lt Lua) -> mlua::Result<mlua::Value<#lt>> {
                    #into_lua_body
                }
            }
        };
    }

    expand.into()
}
