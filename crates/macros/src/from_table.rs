use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

use crate::common::sanitize_field_string;

pub fn from_table(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
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
                                    #field_name: table.get(#field_str).unwrap_or(None),
                                });
                            } else {
                                table_fields.push(quote! {
                                    #field_name: table.get(#field_str)?,
                                });
                            }
                        }
                    }
                }
            }
            _ => {
                panic!("FromTable only works on structs with named fields");
            }
        }
    } else {
        panic!("FromTable only works on structs");
    }

    let table_fields = proc_macro2::TokenStream::from_iter(table_fields);

    let generics = &input.generics.params;

    let expand;

    if generics.is_empty() {
        expand = quote! {
            impl<'a> mlua::FromLua<'a> for #struct_name {
                fn from_lua(value: mlua::Value<'a>, lua: &'a Lua) -> mlua::Result<Self> {
                    if let mlua::Value::Table(table) = value {
                        Ok(Self {
                            #table_fields
                        })
                    } else {
                        Err(mlua::Error::FromLuaConversionError {
                            from: value.type_name(),
                            to: stringify!(#struct_name),
                            message: None,
                        })
                    }

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
            impl<#generics> mlua::FromLua<#lt> for #struct_name {
                fn from_lua(value: mlua::Value<#lt>, lua: &#lt Lua) -> mlua::Result<Self<#generics>> {
                    if let mlua::Value::Table(table) = value {
                        Ok(#struct_name {
                            #table_fields
                        })
                    } else {
                        Err(mlua::Error::FromLuaConversionError {
                            from: value.type_name(),
                            to: stringify!(#struct_name),
                            message: None,
                        })
                    }
                }
            }
        };
    }

    expand.into()
}
