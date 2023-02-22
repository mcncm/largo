extern crate proc_macro;

use model::OptionsData;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};

// The data model (roughly, "IR") of the macro
pub(crate) mod model;
// Error types
pub(crate) mod err;

use err::{Error, Result};

#[proc_macro_derive(Options, attributes(clam))]
pub fn derive_command(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let output = derive_command_inner(derive_input);
    match output {
        Ok(ts) => ts.into(),
        Err(err) => syn::Error::to_compile_error(&err.into()),
    }
    .into()
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

fn derive_command_inner(derive_input: DeriveInput) -> Result<proc_macro2::TokenStream> {
    use darling::FromDeriveInput;
    let OptionsData {
        ident,
        case_convention,
        value_convention,
        data,
    } = OptionsData::from_derive_input(&derive_input)?;
    let fields = match data {
        darling::ast::Data::Struct(fields) => fields,
        darling::ast::Data::Enum(_) => {
            return Err(Error::new(
                derive_input.span(),
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

    Ok(quote::quote! {
        impl clam::Options for #ident {
            fn apply(self, cmd: &mut std::process::Command) {
                #(#apply_by_field)*
            }
        }
    })
}

fn emit_field<F>(field: model::OptionsField, convert_case: F) -> proc_macro2::TokenStream
where
    F: Fn(&str) -> String,
{
    let orig_name = match field.ident {
        Some(ident) => Ok(ident.to_string()),
        None => Err(Error::new(field.ident.span(), "unnamed field")),
    }
    .expect("FIXME: unnamed field; this is actually an internal macro bug");
    let new_name = match field.rename {
        Some(model::Rename(name)) => name,
        None => convert_case(&orig_name),
    };
    quote::quote! {
        clam::ArgValue::set_cmd_arg(#new_name, #orig_name, cmd);
    }
}
