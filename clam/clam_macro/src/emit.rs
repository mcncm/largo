use quote::quote;

use crate::err::{Error, Result};
use crate::model;

pub fn codegen_option_set(option_set: model::OptionsData) -> Result<proc_macro2::TokenStream> {
    Ok(quote!({}))
}
