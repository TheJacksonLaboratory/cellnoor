//! Attribute macros that stamp out the derive groups and SQL impls used
//! throughout `cellnoor-types`.
//!
//! These expand to code that refers to `crate::…`, so they only work inside
//! `cellnoor-types`.

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Fields, Ident, ItemEnum, Type, parse_macro_input};

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
        #[derive(::strum::AsRefStr)]
        #[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
        #[strum(serialize_all = "snake_case")]
    }
}

/// The variants of a predicate enum, each of which holds exactly one value,
/// paired with the type of that value.
fn single_field_variants(item: &ItemEnum) -> Vec<(Ident, Type)> {
    item.variants
        .iter()
        .map(|variant| match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                (variant.ident.clone(), fields.unnamed[0].ty.clone())
            }
            _ => panic!(
                "every variant of a predicate enum holds exactly one value, but `{}` does not",
                variant.ident
            ),
        })
        .collect()
}

fn variant_idents(variants: &[(Ident, Type)]) -> Vec<Ident> {
    variants
        .iter()
        .map(|(variant, _)| variant.clone())
        .collect()
}

/// Turn each variant's operator into the SQL operator and bind value to compare
/// its column against.
///
/// A leaf predicate names its relation through `field_enum`, the enum of its
/// own columns. A wrapper has no columns of its own, so it delegates.
fn as_predicate_impl(
    type_name: &Ident,
    variants: &[Ident],
    field_enum: Option<&Ident>,
) -> proc_macro2::TokenStream {
    let body = match field_enum {
        // `strum(transparent)` makes `as_ref` delegate to the wrapped
        // predicate, so the inner predicate names its own column
        None => quote! {
            match self {
                #(Self::#variants(predicate) => predicate.as_predicate(),)*
            }
        },
        Some(field_enum) => quote! {
            use crate::query::{Field, filter::SqlOperator};

            let (operator, value) = match self {
                #(Self::#variants(operator) => operator.as_sql_operator_and_value(),)*
            };

            crate::query::filter::Predicate {
                relation: <#field_enum as Field>::RELATION,
                column: self.as_ref(),
                operator,
                value,
            }
        },
    };

    quote! {
        #[cfg(feature = "postgres-types")]
        impl crate::query::filter::AsPredicate for #type_name {
            fn as_predicate(&self) -> crate::query::filter::Predicate<'_> {
                #body
            }
        }
    }
}

/// The attribute's argument names the enum of this predicate's columns, as in
/// `#[predicate_enum(InstitutionField)]`. That enum is generated here, and it
/// names the relation every column belongs to.
#[proc_macro_attribute]
pub fn predicate_enum(attr: TokenStream, input: TokenStream) -> TokenStream {
    let field_enum = parse_macro_input!(attr as Ident);
    let enum_derives = enum_derives();

    let item = parse_macro_input!(input as ItemEnum);
    let variants = variant_idents(&single_field_variants(&item));
    let as_predicate = as_predicate_impl(&item.ident, &variants, Some(&field_enum));

    quote! {
        #enum_derives
        #[derive(::strum::EnumDiscriminants)]
        #[strum_discriminants(name(#field_enum), sort_field_enum)]
        #item

        #as_predicate
    }
    .into()
}

#[proc_macro_attribute]
pub fn predicate_enum_wrapper(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let enum_derives = enum_derives();

    let item = parse_macro_input!(input as ItemEnum);
    let wrapper = item.ident.clone();
    let variants = single_field_variants(&item);
    let as_predicate = as_predicate_impl(&wrapper, &variant_idents(&variants), None);

    // Every wrapped predicate converts into the wrapper, and into a filter over
    // the wrapper, so a caller can name an inner predicate and nothing else
    let conversions = variants.iter().map(|(variant, wrapped)| {
        quote! {
            impl From<#wrapped> for #wrapper {
                fn from(predicate: #wrapped) -> Self {
                    Self::#variant(predicate)
                }
            }

            impl From<#wrapped> for crate::query::filter::Filter<#wrapper> {
                fn from(predicate: #wrapped) -> Self {
                    Self::Leaf(predicate.into())
                }
            }
        }
    });

    quote! {
        #enum_derives
        #item

        #as_predicate
        #(#conversions)*
    }
    .into()
}

#[proc_macro_attribute]
pub fn sort_field_enum(_attr: TokenStream, input: TokenStream) -> TokenStream {
    // We can't use enum_derives because strum::EnumDiscriminants already
    // derives most of those traits for us

    let input: proc_macro2::TokenStream = input.into();

    quote! {
        #[derive(Hash, ::strum::AsRefStr, ::strum::IntoStaticStr, ::strum::VariantArray)]
        #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
        #[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
        #[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
        #[strum(serialize_all = "snake_case")]
        #input
    }
    .into()
}

fn enum_sql_impls(module_name: Ident, type_name: Ident) -> proc_macro2::TokenStream {
    quote! {
        #[cfg(feature = "postgres-types")]
        mod #module_name {
            use ::std::str::FromStr;

            use ::bytes::BytesMut;
            use ::postgres_types::{FromSql, ToSql, to_sql_checked};

            use super::#type_name;

            impl<'a> FromSql<'a> for #type_name {
                fn from_sql(
                    ty: &postgres_types::Type,
                    raw: &'a [u8],
                ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
                    let value = <crate::nonempty::NonemptyString as FromSql>::from_sql(ty, raw)?;

                    Ok(Self::from_str(value.as_ref())?)
                }

                fn accepts(ty: &postgres_types::Type) -> bool {
                    <crate::nonempty::NonemptyString as FromSql>::accepts(ty)
                }
            }

            impl ToSql for #type_name {
                to_sql_checked!();

                fn to_sql(
                    &self,
                    ty: &postgres_types::Type,
                    out: &mut BytesMut,
                ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
                where
                    Self: Sized,
                {
                    let value: &str = self.as_ref();
                    <&str as ToSql>::to_sql(&value, ty, out)
                }

                fn accepts(ty: &postgres_types::Type) -> bool
                where
                    Self: Sized,
                {
                    <crate::nonempty::NonemptyString as ToSql>::accepts(ty) || <&str as ToSql>::accepts(ty)
                }
            }
        }
    }
}

#[proc_macro_attribute]
pub fn unit_enum(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let enum_derives = enum_derives();

    let cloned = input.clone();
    let ItemEnum { ident, .. } = parse_macro_input!(cloned as ItemEnum);
    let module_name = format_ident!("postgres_{ident}");

    let input: proc_macro2::TokenStream = input.into();
    let sql_impl_mod = enum_sql_impls(module_name, ident);

    quote! {
        #enum_derives
        #[derive(Copy, Eq, Hash, ::strum::EnumString)]
        #input

        #sql_impl_mod
    }
    .into()
}

#[proc_macro_attribute]
pub fn discriminant_unit_enum(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let cloned = input.clone();
    let ItemEnum { ident, .. } = parse_macro_input!(cloned as ItemEnum);
    let module_name = format_ident!("postgres_{ident}");

    let input: proc_macro2::TokenStream = input.into();
    let sql_impl_mod = enum_sql_impls(module_name, ident);

    quote! {
        #[derive(Hash, ::strum::AsRefStr, ::strum::EnumString, ::strum::VariantArray)]
        #[strum(serialize_all = "snake_case")]
        #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
        #[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
        #[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
        #input

        #sql_impl_mod
    }
    .into()
}
