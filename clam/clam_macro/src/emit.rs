use quote::quote;

use crate::model;
use crate::{Error, Result};

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

struct LoweringCtx {
    convert_case: &'static dyn Fn(&str) -> String,
    _value_convention: model::ValueConvention,
    _array_convention: model::ArrayConvention,
}

impl LoweringCtx {
    fn new(
        case_conv: model::CaseConvention,
        value_conv: model::ValueConvention,
        array_conv: model::ArrayConvention,
    ) -> Self {
        let convert_case: &'static dyn Fn(&str) -> String = match case_conv {
            model::CaseConvention::OneDashKebabCase => &to_one_dash_kebab_case,
            model::CaseConvention::TwoDashKebabCase => &to_two_dash_kebab_case,
        };
        Self {
            convert_case,
            _value_convention: value_conv,
            _array_convention: array_conv,
        }
    }
}

pub fn generate_code(options_data: model::OptionsData) -> Result<proc_macro2::TokenStream> {
    let model::OptionsData {
        ident,
        case_convention,
        value_convention,
        array_convention,
        data,
    } = options_data;
    let ctx = LoweringCtx::new(case_convention, value_convention, array_convention);
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
    // FIXME shouldn't have to dispatch on case convention...
    let apply_by_field = fields.into_iter().map(|field| emit_field(&ctx, field));

    Ok(quote! {
        impl clam::Options for #ident {
            fn apply(self, cmd: &mut std::process::Command) {
                #(#apply_by_field)*
            }
        }
    })
}

fn emit_field(ctx: &LoweringCtx, field: model::OptionsField) -> proc_macro2::TokenStream {
    use syn::spanned::Spanned;
    let orig_name = match field.ident {
        Some(ident) => Ok(ident),
        None => Err(Error::new(field.ident.span(), "unnamed field")),
    }
    .expect("FIXME: unnamed field; this is actually an internal macro bug");
    let new_name = match field.rename {
        Some(model::Rename(name)) => name,
        None => (ctx.convert_case)(&orig_name.to_string()),
    };
    quote! {
        clam::ArgValue::set_cmd_arg(&self.#orig_name, #new_name, cmd);
    }
}
