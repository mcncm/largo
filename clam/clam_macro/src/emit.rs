use quote::quote;

use crate::model;
use crate::{Error, Result};

pub fn generate_code(options_data: model::OptionsData) -> Result<proc_macro2::TokenStream> {
    let model::OptionsData {
        ident,
        case_convention,
        value_convention,
        array_convention,
        data,
    } = options_data;
    let fields = match data {
        darling::ast::Data::Struct(fields) => fields,
        darling::ast::Data::Enum(_) => {
            return Err(Error::new(
                // I know this is the wrong span, but `darling` doesn't save the
                // one for the `enum` keyword
                ident.span(),
                anyhow::anyhow!("can only derive `Command` on a struct"),
            ));
        }
    };

    use model::CaseConvention::*;
    let convert_case = match case_convention {
        OneDashKebabCase => to_one_dash_kebab_case,
        TwoDashKebabCase => to_two_dash_kebab_case,
    };

    // FIXME shouldn't have to dispatch on case convention...
    let apply_by_field = fields
        .into_iter()
        .map(|field| emit_field(field, convert_case));

    Ok(quote! {
        impl clam::Options for #ident {
            fn apply(self, cmd: &mut std::process::Command) {
                #(#apply_by_field)*
            }
        }
    })
}

/// Convert from snake case to kebab_case with one dash
fn to_one_dash_kebab_case(old: &str) -> String {
    std::iter::once('-')
        .chain(old.chars())
        .map(|c| if c == '_' { '_' } else { c })
        .collect()
}

/// Convert from snake case to kebab_case with two dashes
fn to_two_dash_kebab_case(old: &str) -> String {
    std::iter::once('-')
        .chain(std::iter::once('-'))
        .chain(old.chars())
        .map(|c| if c == '_' { '_' } else { c })
        .collect()
}

fn emit_field<F>(field: model::OptionsField, convert_case: F) -> proc_macro2::TokenStream
where
    F: Fn(&str) -> String,
{
    use syn::spanned::Spanned;
    let orig_name = match field.ident {
        Some(ident) => Ok(ident.to_string()),
        None => Err(Error::new(field.ident.span(), "unnamed field")),
    }
    .expect("FIXME: unnamed field; this is actually an internal macro bug");
    let new_name = match field.rename {
        Some(model::Rename(name)) => name,
        None => convert_case(&orig_name),
    };
    quote! {
        clam::ArgValue::set_cmd_arg(#new_name, #orig_name, cmd);
    }
}
