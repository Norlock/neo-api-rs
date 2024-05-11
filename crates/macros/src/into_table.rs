use syn::{parse_macro_input, DeriveInput};
use quote::quote;

pub fn into_table(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;

    let mut table_fields: Vec<proc_macro2::TokenStream> = vec![];

    if let syn::Data::Struct(data_struct) = &input.data {
        match &data_struct.fields {
            syn::Fields::Named(named_fields) => {
                for field in named_fields.named.iter() {
                    let field_name = &field.ident;

                    // TODO recursively
                    if let syn::Type::Path(path_type) = &field.ty {
                        for segment in path_type.path.segments.iter() {
                            let field_type = &segment.ident;
                            if field_type == "Option" {
                                table_fields.push(quote! {
                                    if let Some(value) = self.#field_name {
                                        out.set(stringify!(#field_name), value)?;
                                    }
                                });
                            } else if field_type == "Vec" {
                                table_fields.push(quote! {
                                    let mut lua_arr = lua.create_table()?;

                                    for item in self.#field_name.into_iter() {
                                        lua_arr.push(item)?;
                                    }

                                    out.set(stringify!(#field_name), lua_arr)?;
                                });
                            }  else {
                                table_fields.push(quote! {
                                    out.set(stringify!(#field_name), self.#field_name)?;
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

    let expand = quote! {
        impl<'a> mlua::IntoLua<'a> for #struct_name {
            fn into_lua(self, lua: &'a Lua) -> mlua::Result<mlua::Value<'a>> {
                let out = lua.create_table()?;

                #table_fields

                Ok(mlua::Value::Table(out))
            }
        }
    };

    expand.into()
}
