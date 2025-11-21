use proc_macro::TokenStream;
use quote::quote;
use syn::{Item, parse};

fn base_derives(input: TokenStream, with_default: bool) -> proc_macro2::TokenStream {
    let parsed = parse::<Item>(input).unwrap();
    let serde_default = match &parsed {
        Item::Enum(..) => quote! {},
        Item::Struct(..) => quote! { #[serde(default)] },
        _ => panic!("only enum definitions and struct definitions are supported"),
    };

    if with_default {
        quote! {
            #[derive(Clone, Debug, Default, PartialEq, ::serde::Deserialize, ::serde::Serialize)]
            #[cfg_attr(feature = "typescript", derive(::ts_rs::TS))]
            #serde_default
        }
    } else {
        quote! {
            #[derive(Clone, Debug, PartialEq, ::serde::Deserialize, ::serde::Serialize)]
            #[cfg_attr(feature = "typescript", derive(::ts_rs::TS))]
        }
    }
}

#[proc_macro_attribute]
pub fn base_model(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let derives = base_derives(input.clone(), false);
    let input: proc_macro2::TokenStream = input.into();

    quote! {
        #derives
        #input
    }
    .into()
}

#[proc_macro_attribute]
pub fn base_model_default(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let derives = base_derives(input.clone(), true);
    let input: proc_macro2::TokenStream = input.into();

    quote! {
        #derives
        #input
    }
    .into()
}

fn diesel_insertable() -> proc_macro2::TokenStream {
    quote! {
        #[cfg_attr(feature = "app", derive(::diesel::Insertable))]
    }
}

fn diesel_has_query() -> proc_macro2::TokenStream {
    quote! {
        #[cfg_attr(feature = "app", derive(::diesel::HasQuery))]
    }
}

fn diesel_check_for_backend() -> proc_macro2::TokenStream {
    quote! {
        #[cfg_attr(feature = "app", diesel(check_for_backend(::diesel::pg::Pg)))]
    }
}

#[proc_macro_attribute]
pub fn insert_select(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let base_derives = base_derives(input.clone(), false);
    let insertable = diesel_insertable();
    let has_query = diesel_has_query();
    let check_for_backend = diesel_check_for_backend();

    let input: proc_macro2::TokenStream = input.into();

    quote! {
        #base_derives
        #insertable
        #has_query
        #check_for_backend
        #input
    }
    .into()
}

#[proc_macro_attribute]
pub fn insert(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let base_derives = base_derives(input.clone(), false);
    let insertable = diesel_insertable();
    let check_for_backend = diesel_check_for_backend();

    let input: proc_macro2::TokenStream = input.into();

    quote! {
        #base_derives
        #insertable
        #check_for_backend
        #input
    }
    .into()
}

#[proc_macro_attribute]
pub fn filter(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let base_derives = base_derives(input.clone(), true);

    let input: proc_macro2::TokenStream = input.into();

    quote! {
        #base_derives
        #[cfg_attr(feature = "builder", derive(bon::Builder))]
        #[cfg_attr(feature = "builder", builder(on(_, into)))]
        #input
    }
    .into()
}

#[proc_macro_attribute]
pub fn typescript_query(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let base_derives = base_derives(input.clone(), false);

    let input: proc_macro2::TokenStream = input.into();

    quote! {
        #[cfg(feature = "typescript")]
        #base_derives
        #input
    }
    .into()
}

#[proc_macro_attribute]
pub fn select(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let base_derives = base_derives(input.clone(), false);
    let has_query = diesel_has_query();
    let check_for_backend = diesel_check_for_backend();

    let input: proc_macro2::TokenStream = input.into();

    quote! {
        #base_derives
        #has_query
        #check_for_backend
        #input
    }
    .into()
}

#[proc_macro_attribute]
pub fn update(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let base_derives = base_derives(input.clone(), true);
    let check_for_backend = diesel_check_for_backend();

    let input: proc_macro2::TokenStream = input.into();

    quote! {
        #base_derives
        #[serde(deny_unknown_fields)]
        #[cfg_attr(feature = "app", derive(::diesel::AsChangeset, ::diesel::Identifiable))]
        #check_for_backend
        #input
    }
    .into()
}

#[proc_macro_attribute]
pub fn ordinal_columns(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let base_derives = base_derives(input.clone(), true);

    let input: proc_macro2::TokenStream = input.into();

    quote! {
        #base_derives
        #[derive(Copy)]
        #[serde(rename_all = "snake_case")]
        #input
    }
    .into()
}

#[proc_macro_attribute]
pub fn simple_enum(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let base_derives = base_derives(input.clone(), false);

    let input: proc_macro2::TokenStream = input.into();

    quote! {
        #base_derives
        #[derive(Copy, ::strum::EnumString, ::strum::IntoStaticStr)]
        #[cfg_attr(feature = "app", derive(::diesel::deserialize::FromSqlRow, ::diesel::expression::AsExpression))]
        #[cfg_attr(feature = "app", diesel(sql_type = ::diesel::sql_types::Text))]
        #[serde(rename_all = "snake_case")]
        #[strum(serialize_all = "snake_case")]
        #input
    }.into()
}
