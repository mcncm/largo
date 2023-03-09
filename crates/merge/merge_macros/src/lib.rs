extern crate proc_macro;

use quote::quote;
pub(crate) use syn::Error;
use syn::{parse_macro_input, DeriveInput};

pub(crate) type Result<U> = std::result::Result<U, Error>;

#[proc_macro_derive(Merge, attributes(merge))]
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
    use darling::FromDeriveInput;
    MergeData::from_derive_input(&input)?.emit()
}

/// Attributes on the struct that form the context for how arguments are generated.
#[derive(darling::FromDeriveInput, Debug, Clone)]
#[darling(attributes(merge))]
struct MergeData {
    pub ident: syn::Ident,
    #[allow(unused)]
    pub generics: darling::ast::Generics<()>,
    pub data: darling::ast::Data<darling::util::Ignored, MergeField>,
}

#[derive(darling::FromField, Debug, Clone)]
#[darling(attributes(option))]
struct MergeField {
    pub ident: Option<syn::Ident>,
    pub skip: Option<()>,
}

impl MergeData {
    fn emit(self) -> Result<proc_macro2::TokenStream> {
        let MergeData {
            ident,
            generics: _,
            data,
        } = self;
        let fields = match data {
            darling::ast::Data::Struct(fields) => fields,
            darling::ast::Data::Enum(_) => {
                return Err(Error::new(
                    // I know this is the wrong span, but `darling` doesn't save the
                    // one for the `enum` keyword
                    ident.span(),
                    anyhow::anyhow!("can only derive `Merge` for a struct"),
                ));
            }
        };

        let field_merges_left =
            fields
                .clone()
                .into_iter()
                .filter_map(|field: MergeField| match field.skip {
                    Some(_) => None,
                    None => {
                        let field_ident = field.ident;
                        Some(quote! {
                            self.#field_ident.merge_left(other.#field_ident);
                        })
                    }
                });

        let field_merges_right =
            fields
                .into_iter()
                .filter_map(|field: MergeField| match field.skip {
                    Some(_) => None,
                    None => {
                        let field_ident = field.ident;
                        Some(quote! {
                            self.#field_ident.merge_right(other.#field_ident);
                        })
                    }
                });

        Ok(quote! {
            impl merge::Merge for #ident {
                fn merge_left(&mut self, other: Self) -> &mut Self {
                    #(#field_merges_left)*
                    self
                }

                fn merge_right(&mut self, other: Self) -> &mut Self {
                    #(#field_merges_right)*
                    self
                }
            }
        })
    }
}
