pub fn parse(input: syn::DeriveInput) -> crate::Result<OptionsData> {
    use darling::FromDeriveInput;
    Ok(OptionsData::from_derive_input(&input)?)
}

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

/// How do we format arrays?
#[derive(darling::FromMeta, Debug, Default, Clone)]
#[darling(default)]
pub enum ArrayConvention {
    /// `--param=arg1 --param=arg2 --param=arg3`
    #[default]
    Repeat,
    /// `--param=arg1:arg2:arg3`
    Sep(char),
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
    #[darling(default)]
    pub array_convention: ArrayConvention,
    pub data: darling::ast::Data<darling::util::Ignored, OptionsField>,
}
