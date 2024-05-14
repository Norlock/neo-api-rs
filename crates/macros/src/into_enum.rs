use convert_case::{Case, Casing};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub fn into_enum(input: proc_macro::TokenStream, casing: Option<Case>) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let enum_name = &input.ident;

    fn invalid_enum() {
        // TODO either all variants 1 field or no fields
        // 1 field will use value of field as lua value
        // no field will use to_string() as lua value
        panic!("Enum may not contain any fields, implement it manually");
    }

    let mut variants = Vec::new();

    if let syn::Data::Enum(meta_enum) = &input.data {
        for variant in meta_enum.variants.iter() {
            if !variant.fields.is_empty() {
                invalid_enum();
            }

            let variant = &variant.ident;

            let variant_value = if let Some(casing) = casing {
                variant.to_string().to_case(casing)
            } else {
                variant.to_string()
            };

            variants.push(quote! {
                Self::#variant => f.write_str(#variant_value),
            });
        }
    }

    let variants_stream = proc_macro2::TokenStream::from_iter(variants);

    let expand = quote! {
        impl std::fmt::Display for #enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #variants_stream
                }
            }
        }

        impl<'a> mlua::IntoLua<'a> for #enum_name {
            fn into_lua(self, lua: &'a Lua) -> mlua::Result<mlua::Value<'a>> {
                let str = lua.create_string(self.to_string())?;

                Ok(mlua::Value::String(str))
            }
        }
    };

    expand.into()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[derive(IntoTable)]
    pub struct SomeStruct {
        pub name: String,
        pub age: usize,
        pub weight: Option<usize>,
        pub phone: Phone,
        pub gender: Gender,
    }

    #[derive(IntoTable)]
    pub struct Phone {
        pub mobile: String,
        pub home: String,
    }

    #[derive(IntoEnumSC)]
    pub enum Gender {
        Male,
        Female,
    }

    #[test]
    fn test_macro() {
        // TODO write test
    }
}
