//! This module is probably not supposed to be here; I'm guessing the "right"
//! way to do this is to implement a bunch of `syn::Parse` traits

use anyhow::anyhow;
use syn::parse::{Parse, ParseStream};
use syn::token::{Enum, Union};
use syn::{
    parse_macro_input, Attribute, Data, DataEnum, DataStruct, DataUnion, DeriveInput, Fields,
    Generics, Ident, PathArguments, PathSegment,
};

use crate::err::{Error, Result};
use crate::model;

pub const ATTR_PREFIX: &'static str = "clam";
pub const ATTR_OPTION: &'static str = "clam";

pub struct OptionsInput {
    attrs: Vec<Attribute>,
    generics: Generics,
    fields: Fields,
}

impl TryFrom<DeriveInput> for OptionsInput {
    type Error = Error;

    fn try_from(input: DeriveInput) -> Result<Self> {
        let DeriveInput {
            attrs,
            generics,
            data,
            ..
        } = input;

        match data {
            Data::Struct(DataStruct { fields, .. }) => Ok(OptionsInput {
                attrs,
                generics,
                fields,
            }),
            Data::Enum(DataEnum {
                enum_token: Enum { span },
                ..
            })
            | Data::Union(DataUnion {
                union_token: Union { span },
                ..
            }) => Err(Error::new(
                span,
                anyhow!("can only derive `Options` on a `struct`"),
            )),
        }
    }
}

pub fn parse_option_set(input: DeriveInput) -> Result<model::OptionSet> {
    let OptionsInput {
        attrs,
        generics,
        fields,
    } = OptionsInput::try_from(input)?;
    parse_option_set_inner(attrs, generics, fields)
}

fn parse_option_set_inner(
    attrs: Vec<Attribute>,
    generics: Generics,
    fields: Fields,
) -> Result<model::OptionSet> {
    Ok(model::OptionSet {})
}

impl TryFrom<Vec<Attribute>> for model::OptionsAttrs {
    type Error = Error;

    fn try_from(outer_attrs: Vec<Attribute>) -> Result<Self> {
        for Attribute { path, tokens, .. } in outer_attrs {
            // Try to match `#[clam(...)]`; if you do, update with the inside
            if !path.is_ident(ATTR_PREFIX) {
                continue;
            }

            return syn::parse2::<model::OptionsAttrs>(tokens);
        }
        Ok(model::OptionsAttrs::default())
    }
}

impl Parse for model::OptionsAttrs {
    fn parse(mut input: ParseStream) -> syn::Result<Self> {
        let mut options_attrs = model::OptionsAttrs::default();
        while !input.is_empty() {
            options_attrs.parse_next(&mut input)?;
        }
        Ok(options_attrs)
    }
}

impl model::OptionsAttrs {
    fn parse_next(&mut self, input: &mut ParseStream) -> Result<()> {
        let lookahead = input.lookahead1();
        // if lookahead.peek(Token![]);
        Ok(())
    }
}

impl Parse for model::CaseConvention {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        todo!()
    }
}

impl Parse for model::ValueConvention {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        todo!()
    }
}

impl TryFrom<Ident> for model::CaseConvention {
    type Error = Error;

    fn try_from(ident: Ident) -> Result<Self> {
        match ident.to_string().as_str() {
            "one_dash_kebab_case" => Ok(Self::OneDashKebabCase),
            "two_dash_kebab_case" => Ok(Self::TwoDashKebabCase),
            other => Err(Error::new(
                ident.span(),
                anyhow!("unknown case convention `{}`", other),
            )),
        }
    }
}

impl TryFrom<Ident> for model::ValueConvention {
    type Error = Error;

    fn try_from(ident: Ident) -> Result<Self> {
        match ident.to_string().as_str() {
            "space" => Ok(Self::Space),
            "no_space_equals" => Ok(Self::NoSpaceEquals),
            other => Err(Error::new(
                ident.span(),
                anyhow!("unknown value convention `{}`", other),
            )),
        }
    }
}
