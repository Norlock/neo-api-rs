use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub fn from_table(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;

    let mut table_fields: Vec<proc_macro2::TokenStream> = vec![];

    proc_macro::TokenStream::new()

}
