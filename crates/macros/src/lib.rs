use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(IntoLua)]
pub fn tag_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;

    let mut table_fields: Vec<proc_macro2::TokenStream> = vec![];

    if let syn::Data::Struct(data_struct) = &input.data {
        match &data_struct.fields {
            //syn::Fields::Unnamed(unnamed_fields) => {
                //for field in unnamed_fields.unnamed.iter() {
                    //match field.ty {
                        //syn::Type::Path(path) => {
                            //for segment in path.path.segments.iter() {
                                //if segment.ident.to_string() == "Options".to_string() {
                                    //table_fields.push(quote! {
                                        //if let Some(value) = 
                                    //})
                                //}
                            //}
                        //}
                    //}
                //}
            //}
            syn::Fields::Named(named_fields) => {
                for field in named_fields.named.iter() {
                    let field_name = &field.ident;

                    if let syn::Type::Path(path_type) = &field.ty {
                        //if path

                    }
                    table_fields.push(quote! {
                        out.set(stringify!(#field_name), self.#field_name)?;
                    });

                }
            }
            _ => {}
        }

    } else {
        panic!("EnumDeserialize only works on enums");
    }

    let table_fields = proc_macro2::TokenStream::from_iter(table_fields);

    let expand = quote! {
        impl<'a> IntoLua<'a> for #struct_name {
            fn into_lua(self, lua: &'a Lua) -> LuaResult<LuaValue<'a>> {
                let out = lua.create_table()?;

                #table_fields

                Ok(LuaValue::Table(out))
            }
        }
    };

    expand.into()
}
