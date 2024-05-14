use convert_case::Case;

mod into_enum;
mod into_table;
mod from_table;
mod common;

/// A simple into table function (yet), for complex structures create your own into lua function
#[proc_macro_derive(IntoTable)]
pub fn into_table(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    into_table::into_table(input)
}

/// A simple into table function (yet), for complex structures create your own into lua function
#[proc_macro_derive(FromTable)]
pub fn from_table(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    from_table::from_table(input)
}

/// Into enum (copy field name to string)
/// Also implements Display
#[proc_macro_derive(IntoEnum)]
pub fn into_enum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    into_enum::into_enum(input, None)
}

/// Into enum camel case
/// Also implements Display
#[proc_macro_derive(IntoEnumCC)]
pub fn into_enum_cc(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    into_enum::into_enum(input, Some(Case::Camel))
}

/// Into enum snake case
/// Also implements Display
#[proc_macro_derive(IntoEnumSC)]
pub fn into_enum_sc(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    into_enum::into_enum(input, Some(Case::Snake))
}

/// Into enum upper case
/// Also implements Display
#[proc_macro_derive(IntoEnumUC)]
pub fn into_enum_uc(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    into_enum::into_enum(input, Some(Case::Upper))
}

/// Into enum pascal case
/// Also implements Display
#[proc_macro_derive(IntoEnumPC)]
pub fn into_enum_pc(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    into_enum::into_enum(input, Some(Case::Pascal))
}
