//! Clam (command-line argument maker) is the opposite of Clap.

// Ensure that `clam::SomeItem` resolves to the right item within expanded code
extern crate self as clam;

extern crate proc_macro;

use syn::{parse_macro_input, DeriveInput};

/// Code generation
pub(crate) mod emit;
/// The data model (roughly, "IR") of the macro
pub(crate) mod model;

pub(crate) use syn::Error;

pub(crate) type Result<U> = std::result::Result<U, Error>;

#[proc_macro_derive(Options, attributes(clam))]
pub fn derive_command(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // This has to go here: `parse_macro_input!` requires `proc_macro::TokenStream` return type.
    let derive_input = parse_macro_input!(input as DeriveInput);
    let output = derive_command_inner(derive_input);
    match output {
        Ok(ts) => ts.into(),
        Err(err) => syn::Error::to_compile_error(&err.into()),
    }
    .into()
}

fn derive_command_inner(input: DeriveInput) -> Result<proc_macro2::TokenStream> {
    let ir = model::parse(input)?;
    emit::generate_code(ir)
}
