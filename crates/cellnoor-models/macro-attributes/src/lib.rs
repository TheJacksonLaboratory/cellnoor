use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, ItemEnum, parse_macro_input};

fn base_derives(with_default: bool) -> proc_macro2::TokenStream {
    if with_default {
        quote! {
            #[derive(Clone, Debug, Default, PartialEq)]
        }
    } else {
        quote! {
            #[derive(Clone, Debug, PartialEq)]
        }
    }
}

#[proc_macro_attribute]
pub fn base_model(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let derives = base_derives(false);
    let input: proc_macro2::TokenStream = input.into();

    quote! {
        #derives
        #input
    }
    .into()
}

#[proc_macro_attribute]
pub fn base_model_default(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let derives = base_derives(true);
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

fn builder() -> proc_macro2::TokenStream {
    quote! {
        #[cfg_attr(feature = "builder", derive(bon::Builder))]
        #[cfg_attr(feature = "builder", builder(on(_, into)))]
    }
}

#[proc_macro_attribute]
pub fn insert_select(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let base_derives = base_derives(false);
    let insertable = diesel_insertable();
    let has_query = diesel_has_query();
    let check_for_backend = diesel_check_for_backend();
    let builder = builder();

    let input: proc_macro2::TokenStream = input.into();

    quote! {
        #[derive(::serde::Deserialize, ::serde::Serialize)]
        #[cfg_attr(feature = "app", derive(::schemars::JsonSchema))]
        #base_derives
        #insertable
        #has_query
        #[serde(deny_unknown_fields)]
        #check_for_backend
        #builder
        #input
    }
    .into()
}

#[proc_macro_attribute]
pub fn insert(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let base_derives = base_derives(false);
    let insertable = diesel_insertable();
    let check_for_backend = diesel_check_for_backend();
    let builder = builder();

    let input: proc_macro2::TokenStream = input.into();

    quote! {
        #[derive(::serde::Deserialize)]
        #[cfg_attr(feature = "app", derive(::schemars::JsonSchema))]
        #base_derives
        #insertable
        #[serde(deny_unknown_fields)]
        #check_for_backend
        #builder
        #input
    }
    .into()
}

#[proc_macro_attribute]
pub fn filter(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let base_derives = base_derives(true);

    let input: proc_macro2::TokenStream = input.into();

    quote! {
        #[derive(::serde::Deserialize, ::serde::Serialize)]
        #[cfg_attr(feature = "app", derive(::schemars::JsonSchema))]
        #base_derives
        #[serde(deny_unknown_fields)]
        #[cfg_attr(feature = "builder", derive(bon::Builder))]
        #[cfg_attr(feature = "builder", builder(on(_, into)))]
        #input
    }
    .into()
}

#[proc_macro_attribute]
pub fn select(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let base_derives = base_derives(false);
    let has_query = diesel_has_query();
    let check_for_backend = diesel_check_for_backend();

    let input: proc_macro2::TokenStream = input.into();

    quote! {
        #[derive(::serde::Serialize)]
        #[cfg_attr(feature = "app", derive(::schemars::JsonSchema))]
        #base_derives
        #has_query
        #[serde(deny_unknown_fields)]
        #check_for_backend
        #input
    }
    .into()
}

#[proc_macro_attribute]
pub fn update(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let base_derives = base_derives(true);
    let check_for_backend = diesel_check_for_backend();

    let input: proc_macro2::TokenStream = input.into();

    quote! {
        #[derive(::serde::Deserialize)]
        #[cfg_attr(feature = "app", derive(::schemars::JsonSchema))]
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
    let base_derives = base_derives(false);

    let cellnoor_schema_mod = parse_macro_input!(attr as Ident);
    let diesel_mod = format_ident!("diesel_{cellnoor_schema_mod}");
    let enum_def = parse_macro_input!(input as ItemEnum);

    let enum_name = &enum_def.ident;

    let items = enum_def.variants.iter().map(|v| {
        let v = &v.ident;

        let asc_static = format_ident!("asc_{v}");
        let desc_static = format_ident!("desc_{v}");

        (v, asc_static, desc_static)
    });

    let first_field = items.clone().map(|(v, ..)| v).next().unwrap();

    let static_defs = items.clone().map(|(v, asc_static, desc_static)| {
        quote! {
            #[allow(non_upper_case_globals)]
            static #asc_static: LazyLock<Asc<#cellnoor_schema_mod::#v>> = LazyLock::new(|| #cellnoor_schema_mod::#v.asc());
            static #desc_static: LazyLock<Desc<#cellnoor_schema_mod::#v>> = LazyLock::new(|| #cellnoor_schema_mod::#v.desc());
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
        #[derive(Copy, Eq, PartialOrd, Ord, Hash, ::serde::Deserialize, ::serde::Serialize)]
        #[cfg_attr(feature = "app", derive(::schemars::JsonSchema))]
        #[serde(tag = "field", rename_all = "snake_case")]
        #enum_def

        #[cfg(feature = "app")]
        mod #diesel_mod {
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

            impl<T> AppearsOnTable<T> for #enum_name where #cellnoor_schema_mod::#first_field: AppearsOnTable<T> {}

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
    let base_derives = base_derives(false);

    let input: proc_macro2::TokenStream = input.into();

    quote! {
        #base_derives
        #[derive(Copy, Eq, PartialOrd, Ord, Hash, ::strum::EnumString, ::strum::IntoStaticStr, ::serde::Deserialize, ::serde::Serialize)]
        #[cfg_attr(feature = "app", derive(::schemars::JsonSchema))]
        #[cfg_attr(feature = "app", derive(::diesel::deserialize::FromSqlRow, ::diesel::expression::AsExpression))]
        #[cfg_attr(feature = "app", diesel(sql_type = ::diesel::sql_types::Text, sql_type = ::cellnoor_schema::sql_types::CaseInsensitiveText))]
        #[serde(rename_all = "snake_case")]
        #[strum(serialize_all = "snake_case")]
        #input
    }.into()
}

#[proc_macro_attribute]
pub fn json(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let base_derives = base_derives(false);

    let input: proc_macro2::TokenStream = input.into();

    quote! {
        #base_derives
        #[derive(::serde::Deserialize, ::serde::Serialize)]
        #[cfg_attr(feature = "app", derive(::schemars::JsonSchema))]
        #[cfg_attr(feature = "app", derive(::diesel::deserialize::FromSqlRow, ::diesel::expression::AsExpression))]
        #[cfg_attr(feature = "app", diesel(sql_type = ::diesel::sql_types::Jsonb))]
        #[serde(rename_all = "snake_case")]
        #input
    }.into()
}
