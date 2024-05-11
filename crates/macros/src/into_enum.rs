use convert_case::{Case, Casing};
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

pub fn into_enum(input: proc_macro::TokenStream, casing: Case) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let enum_name = &input.ident;

    fn invalid_enum() {
        panic!("Enum my not contain any fields (impl IntoLua manually)");
    }

    let mut variants = Vec::new();

    if let syn::Data::Enum(meta_enum) = &input.data {
        for variant in meta_enum.variants.iter() {
            if !variant.fields.is_empty() {
                invalid_enum();
            }

            let variant = &variant.ident;
            let variant_value = format_ident!("{}", variant.to_string().to_case(casing));

            variants.push(quote! {
                Self::#variant => f.write_str(stringify!(#variant_value)),
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
