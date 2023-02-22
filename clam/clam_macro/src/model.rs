/// How do we construct flags from field names?
#[derive(darling::FromMeta, Debug, Default, Clone)]
#[darling(default)]
pub enum CaseConvention {
    /// `-my-param`
    OneDashKebabCase,
    /// `--my-param`
    #[default]
    TwoDashKebabCase,
}

/// How do we assign values to parameters?
#[derive(darling::FromMeta, Debug, Default, Clone)]
#[darling(default)]
pub enum ValueConvention {
    /// `--param arg`
    #[default]
    Space,
    /// `--param=arg`
    NoSpaceEquals,
}

#[derive(darling::FromMeta, Debug, Clone)]
pub struct Rename(pub String);

#[derive(darling::FromField, Debug, Clone)]
#[darling(attributes(option))]
pub struct OptionsField {
    pub ident: Option<syn::Ident>,
    #[darling(default)]
    pub rename: Option<Rename>,
}

/// Attributes on the struct that form the context for how arguments are generated.
/// For example,
#[derive(darling::FromDeriveInput, Debug, Clone)]
#[darling(attributes(clam))]
pub struct OptionsData {
    pub ident: syn::Ident,
    #[darling(default)]
    pub case_convention: CaseConvention,
    #[darling(default)]
    pub value_convention: ValueConvention,
    pub data: darling::ast::Data<darling::util::Ignored, OptionsField>,
}
