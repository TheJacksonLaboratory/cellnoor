use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemEnum, parse_macro_input};

fn base_derives() -> proc_macro2::TokenStream {
    quote! {
        #[derive(Clone, Debug, PartialEq)]
        #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
        #[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]

    }
}

#[proc_macro_attribute]
pub fn base_model(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let derives = base_derives();
    let input: proc_macro2::TokenStream = input.into();

    quote! {
        #derives
        #input
    }
    .into()
}

#[proc_macro_attribute]
pub fn select(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let base_derives = base_derives();

    let input: proc_macro2::TokenStream = input.into();

    quote! {
        #base_derives
        #[cfg_attr(feature = "postgres-types", derive(postgres_types::FromSql))]
        #input
    }
    .into()
}

fn enum_derives() -> proc_macro2::TokenStream {
    let base_derives = base_derives();

    quote! {
        #base_derives
        #[derive(Copy, Eq, Hash, ::strum::AsRefStr)]
        #[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
        #[strum(serialize_all = "snake_case")]
    }
}

#[proc_macro_attribute]
pub fn field_enum(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let enum_derives = enum_derives();

    let input: proc_macro2::TokenStream = input.into();

    quote! {
        #enum_derives
        #input
    }
    .into()
}

#[proc_macro_attribute]
pub fn unit_enum(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let enum_derives = enum_derives();

    let cloned = input.clone();
    let ItemEnum { ident, .. } = parse_macro_input!(cloned as ItemEnum);
    let module_name = format_ident!("postgres_{ident}");

    let input: proc_macro2::TokenStream = input.into();

    quote! {
        #enum_derives
        #[derive(::strum::EnumString)]
        #input

        #[cfg(feature = "postgres-types")]
        mod #module_name {
            use std::str::FromStr;

            use bytes::BytesMut;
            use postgres_types::{FromSql, ToSql, to_sql_checked};

            use super::#ident;

            impl<'a> FromSql<'a> for #ident {
                fn from_sql(
                    ty: &postgres_types::Type,
                    raw: &'a [u8],
                ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
                    Ok(<nonempty::NonemptyString as FromSql>::from_sql(ty, raw)
                        .map(|s| Self::from_str(s.as_ref()))
                        .unwrap()
                        .unwrap())
                }

                fn accepts(ty: &postgres_types::Type) -> bool {
                    <nonempty::NonemptyString as FromSql>::accepts(ty)
                }
            }

            impl ToSql for #ident {
                to_sql_checked!();

                fn to_sql(
                    &self,
                    ty: &postgres_types::Type,
                    out: &mut BytesMut,
                ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
                where
                    Self: Sized,
                {
                    nonempty::NonemptyString::new(self.as_ref().to_owned()).unwrap().to_sql(ty, out)
                }

                fn accepts(ty: &postgres_types::Type) -> bool
                where
                    Self: Sized,
                {
                    <nonempty::NonemptyString as ToSql>::accepts(ty)
                }
            }
        }


    }
    .into()
}
