use convert_case::Case;

mod into_enum;
mod into_table;

#[proc_macro_derive(IntoTable)]
pub fn into_table(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    into_table::into_table(input)
}

/// Into enum camel case
#[proc_macro_derive(IntoEnumCC)]
pub fn into_enum_cc(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    into_enum::into_enum(input, Case::Camel)
}

/// Into enum snake case
#[proc_macro_derive(IntoEnumSC)]
pub fn into_enum_sc(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    into_enum::into_enum(input, Case::Snake)
}

/// Into enum upper case
#[proc_macro_derive(IntoEnumUC)]
pub fn into_enum_uc(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    into_enum::into_enum(input, Case::Upper)
}

/// Into enum pascal case
#[proc_macro_derive(IntoEnumPC)]
pub fn into_enum_pc(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    into_enum::into_enum(input, Case::Pascal)
}
