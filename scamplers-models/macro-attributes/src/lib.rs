use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Item, ItemEnum, Path, parse, parse_macro_input};

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
            #[cfg_attr(feature = "typescript", ts(optional_fields))]
            #serde_default
        }
    } else {
        quote! {
            #[derive(Clone, Debug, PartialEq, ::serde::Deserialize, ::serde::Serialize)]
            #[cfg_attr(feature = "typescript", derive(::ts_rs::TS))]
            #[cfg_attr(feature = "typescript", ts(optional_fields))]
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
        #[serde(deny_unknown_fields)]
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
pub fn order_by(attr: TokenStream, input: TokenStream) -> TokenStream {
    let base_derives = base_derives(input.clone(), false);

    let scamplers_schema_mod = parse_macro_input!(attr as Path);
    let enum_def = parse_macro_input!(input as ItemEnum);

    let enum_name = &enum_def.ident;

    let items = enum_def.variants.iter().map(|v| {
        let v = &v.ident;

        let asc_static = format_ident!("asc_{v}");
        let desc_static = format_ident!("desc_{v}");

        (v, asc_static, desc_static)
    });

    let static_defs = items.clone().map(|(v, asc_static, desc_static)| {
        quote! {
            #[allow(non_upper_case_globals)]
            static #asc_static: LazyLock<Asc<#scamplers_schema_mod::#v>> = LazyLock::new(|| #scamplers_schema_mod::#v.asc());
            static #desc_static: LazyLock<Desc<#scamplers_schema_mod::#v>> = LazyLock::new(|| #scamplers_schema_mod::#v.desc());
        }
    });

    let match_bodies = items.map(|(v, asc_static, desc_static)| {
        quote! {
            Self::#v { descending: None | Some(false) } => #asc_static.walk_ast(pass),
            Self::#v { descending: Some(true) } => #desc_static.walk_ast(pass),
        }
    });

    quote! {
        #base_derives
        #[derive(Copy)]
        #[serde(rename_all = "snake_case", tag = "field")]
        #enum_def

        #[cfg(feature = "app")]
        mod diesel_impl {
            use ::std::sync::LazyLock;

            use super::*;
            use ::diesel::{
                dsl::{Asc, Desc},
                expression::expression_types::NotSelectable,
                pg::Pg,
                prelude::*,
                query_builder::QueryFragment,
            };

            #(#static_defs)*

            impl Expression for #enum_name {
                type SqlType = NotSelectable;
            }

            impl AppearsOnTable<#scamplers_schema_mod::table> for #enum_name {}

            impl QueryFragment<Pg> for #enum_name {
                fn walk_ast<'b>(
                    &'b self,
                    pass: diesel::query_builder::AstPass<'_, 'b, Pg>,
                ) -> diesel::QueryResult<()> {
                    match self {
                        #(#match_bodies)*
                    }
                }
            }
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn simple_enum(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let base_derives = base_derives(input.clone(), false);

    let input: proc_macro2::TokenStream = input.into();

    quote! {
        #base_derives
        #[derive(Copy, Eq, PartialOrd, Ord, ::strum::EnumString, ::strum::IntoStaticStr)]
        #[cfg_attr(feature = "app", derive(::diesel::deserialize::FromSqlRow, ::diesel::expression::AsExpression))]
        #[cfg_attr(feature = "app", diesel(sql_type = ::diesel::sql_types::Text))]
        #[serde(rename_all = "snake_case")]
        #[strum(serialize_all = "snake_case")]
        #input
    }.into()
}

#[proc_macro_attribute]
pub fn json(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let base_derives = base_derives(input.clone(), false);

    let input: proc_macro2::TokenStream = input.into();

    quote! {
        #base_derives
        #[cfg_attr(feature = "app", derive(::diesel::deserialize::FromSqlRow, ::diesel::expression::AsExpression))]
        #[cfg_attr(feature = "app", diesel(sql_type = ::diesel::sql_types::Jsonb))]
        #[serde(rename_all = "snake_case")]
        #input
    }.into()
}
