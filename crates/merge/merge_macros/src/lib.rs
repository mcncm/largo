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
    ident: syn::Ident,
    #[allow(unused)]
    generics: syn::Generics,
    data: darling::ast::Data<darling::util::Ignored, MergeField>,
    #[darling(default)]
    replace: bool,
}

#[derive(darling::FromField, Debug, Clone)]
#[darling(attributes(option))]
struct MergeField {
    ident: Option<syn::Ident>,
    skip: Option<()>,
}

impl MergeData {
    fn emit(self) -> Result<proc_macro2::TokenStream> {
        let MergeData {
            ident,
            generics:
                syn::Generics {
                    params,
                    where_clause,
                    ..
                },
            data,
            replace,
        } = self;
        let impls = if replace {
            quote! {
                fn merge_left(&mut self, other: Self) -> &mut Self {
                    self
                }

                fn merge_right(&mut self, other: Self) -> &mut Self {
                    *self = other;
                    self
                }
            }
        } else {
            let fields = match data {
                darling::ast::Data::Struct(fields) => fields,
                darling::ast::Data::Enum(_) => {
                    return Err(Error::new(
                        // I know this is the wrong span, but `darling` doesn't save the
                        // one for the `enum` keyword
                        ident.span(),
                        anyhow::anyhow!(
                            "must use `#[merge(replace)]` to derive `Merge` for an enum"
                        ),
                    ));
                }
            };

            emit_impls_rec(fields)
        };
        Ok(quote! {
            impl<#params> merge::Merge for #ident<#params> #where_clause {
                #impls
            }
        })
    }
}

fn emit_impls_rec(fields: darling::ast::Fields<MergeField>) -> proc_macro2::TokenStream {
    let field_merges_left =
        fields
            .clone()
            .into_iter()
            .enumerate()
            .filter_map(|(idx, field): (usize, MergeField)| {
                let idx = syn::Index::from(idx);
                match (field.skip, field.ident) {
                    // This field is skipped
                    (Some(_), _) => None,
                    // This is a tuple field
                    (_, None) => Some(quote! {
                            merge::Merge::merge_left(&mut self.#idx, other.#idx);
                    }),
                    // This is a named field
                    (_, Some(ident)) => Some(quote! {
                            merge::Merge::merge_left(&mut self.#ident, other.#ident);
                    }),
                }
            });

    let field_merges_right =
        fields
            .into_iter()
            .enumerate()
            .filter_map(|(idx, field): (usize, MergeField)| {
                let idx = syn::Index::from(idx);
                match (field.skip, field.ident) {
                    // This field is skipped
                    (Some(_), _) => None,
                    // This is a tuple field
                    (_, None) => Some(quote! {
                            merge::Merge::merge_right(&mut self.#idx, other.#idx);
                    }),
                    // This is a named field
                    (_, Some(ident)) => Some(quote! {
                            merge::Merge::merge_right(&mut self.#ident, other.#ident);
                    }),
                }
            });

    quote! {
        fn merge_left(&mut self, other: Self) -> &mut Self {
            #(#field_merges_left)*
            self
        }

        fn merge_right(&mut self, other: Self) -> &mut Self {
            #(#field_merges_right)*
            self
        }
    }
}
