#![allow(unused)]

use crate::Result;

use serde::Deserialize;

pub type Year = i32;

pub type Lang = String;

pub type RelativePath = String;

pub type PackageId = String;

/// A package from the CTAN database as queried with the JSON api and specified
/// at [this](https://ctan.org/help/json/2.0/pkg) page.
#[derive(Debug, Clone, Deserialize)]
pub struct Package {
    /// This attribute contains the unique id of the package. This attribute is
    /// mandatory.
    pub id: PackageId,

    /// This attribute contains a list of aliases for the package. The alias is
    /// a object which has several attributes:
    #[serde(default)]
    pub aliases: Vec<Alias>,

    /// The entry has the mandatory attribute <name>. The name contains the
    /// print representation of the package name.
    pub name: String,

    /// The entry has the mandatory attribute <caption>. The caption contains a
    /// short description of the package.
    pub caption: String,

    /// The entry has the attribute authors which contains a list of authors.
    /// The author is a object which has several attributes:
    #[serde(default)]
    pub authors: Vec<Author>,

    /// The entry can have a list-valued attribute copyright. It carries the
    /// information about the copyright. This list contains objects which have
    /// several attributes:
    #[serde(default)]
    pub copyright: Vec<Copyright>,

    /// The entry can have an attribute license.
    pub license: License,

    /// The entry has the attribute version. It carries the information about
    /// the version of the package. This object has several attributes.
    pub version: Version,

    /// The entry has a list of description objects. It may have attributes:
    #[serde(default)]
    pub descriptions: Vec<Description>,

    /// An inner tag of <description> is <ref>. It is used to reference a
    /// package. The tag may have an attribute:
    ///
    /// NOTE: I am pretty sure that this isn't *actually* a field in the json
    /// object, despite what the spec says. For now I'm wrapping it in an
    /// `Option` and skipping it.
    #[serde(rename = "<ref>")]
    #[serde(skip)]
    r#ref: Option<DescriptionRef>,

    /// The entry has the list attribute documentation. The list elements
    /// indicate references to documentation. The objects may have attributes:
    #[serde(default)]
    pub documentation: Vec<Documentation>,

    /// The entry has the optional attribute ctan. It carries the location of
    /// the package in the CTAN tree. This JSON object has several attributes:
    #[serde(default)]
    pub ctan: Option<CtanLocation>,

    /// The entry has the optional attribute install. It carries the location of
    /// the package on CTAN in form of an installable TDS-compliant zip archive.
    /// This JSON object has several attributes.
    #[serde(default)]
    pub install: Option<Install>,

    /// The entry has the optional attribute miktex. It carries the name of the
    /// package in MiKTEX. This JSON object has several attributes.
    #[serde(default)]
    pub miktex: Option<Miktex>,

    /// The entry has the optional attribute texlive. It carries the name of the
    /// package in TEX Live. This JSON object has several attributes:
    #[serde(default)]
    pub texlive: Option<Texlive>,

    /// The entry has the optional attribute index. If present then it contains
    /// a list of extra terms to be indexed for the search.
    #[serde(default)]
    pub index: Option<Vec<String>>,

    /// The entry has the optional attribute topics. If present then it contains
    /// a list of topics keys for this entry.
    #[serde(default)]
    pub topics: Option<Vec<String>>,

    /// The entry has the optional attribute home. If present then it contains
    /// the URL of the home page of the package.
    #[serde(default)]
    pub home: Option<String>,

    /// The entry has the optional attribute support. If present then it
    /// contains the URL of the support for the package.
    #[serde(default)]
    pub support: Option<String>,

    /// The entry has the optional attribute announce. If present then it
    /// contains the URL of the announcements for the package.
    #[serde(default)]
    pub announce: Option<String>,

    /// The entry has the optional attribute bugs. If present then it contains
    /// the URL of the bug tracker for the package.
    #[serde(default)]
    pub bugs: Option<String>,

    /// The entry has the optional attribute repository. If present then it
    /// contains the URL of the source code repository for the package.
    #[serde(default)]
    pub repository: Option<String>,

    /// The entry has the optional attribute development. If present then it
    /// contains the URL of the developer community for the package.
    #[serde(default)]
    pub development: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Alias {
    /// This attribute contains the id of the alias. This attribute is mandatory.
    pub id: PackageId,
    /// This attribute is the name of the alias. It is mandatory.
    pub name: String,
}

pub type AuthorId = String;

#[derive(Debug, Clone, Deserialize)]
pub struct Author {
    /// This attribute contains the id of the author. This attribute is
    /// mandatory.
    pub id: AuthorId,

    /// This attribute is the title of the author. It is optional and can be
    /// empty. The default is empty.
    #[serde(default)]
    pub title: Option<String>,

    /// This attribute contains the given name. It is optional and can be empty.
    #[serde(default)]
    pub givenname: Option<String>,

    /// This attribute is the von part of the author's name. It is usually in
    /// lower case and has values like von, van, or de. It is optional and can
    /// be empty. The default is empty.
    #[serde(default)]
    pub von: Option<String>,

    /// This attribute is the family name. It is optional and can be empty.
    #[serde(default)]
    pub familyname: Option<String>,

    /// This attribute is the junior part of the author's name. It is usually an
    /// addition to the name like jr., sr., or a numeral like I, II, III, IV. It
    /// is optional and can be empty. The default is empty.
    #[serde(default)]
    pub junior: Option<String>,

    /// This attribute is the alias name to protect the privacy of an author who
    /// requests it. It is optional and can be empty. The default is empty. In
    /// case this attribute is not empty the other name constituents are not
    /// shown.
    #[serde(default)]
    pub pseudonym: Option<String>,

    /// This attribute is the boolean indicator that the author is female. It is
    /// optional and can be empty. The default is false.
    ///
    /// NOTE: I didn't write this spec!!!
    #[serde(default)]
    pub female: bool,

    /// This attribute is the indicator that the author is deceased. It is
    /// optional and can be empty. The default is empty.
    #[serde(default)]
    pub died: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Copyright {
    /// This attribute contains the name of the copyright holder. This attribute
    /// is mandatory.
    pub owner: String,

    /// This attribute contains the year or years of the copyright. This
    /// attribute is mandatory.
    ///
    /// NOTE: Although this attribvute is described as mandatory, it's absent
    /// even from the example package (`tex`) described in the API doc!
    pub year: Option<Year>,
}

/// At least one of number or date have to be given. Otherwise the tag is
/// suppressed.
#[derive(Debug, Clone, Deserialize)]
pub struct Version {
    /// This attribute contains the version number.
    #[serde(default)]
    pub number: Option<String>,

    /// This attribute contains the version date.
    #[serde(default)]
    pub date: Option<String>,
}

/// The value can be either a string or a list of strings with keys of licenses.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum License {
    String(String),
    /// FIXME: I don't really know what they want here.
    List(Vec<String>),
}

#[derive(Debug, Clone, Deserialize)]
pub struct Description {
    /// This attribute contains the longer description of the package. It may include HTML markup.
    ///
    /// NOTE: the API spec says that this field should be called `description`,
    /// but the very example package (`tex`) described by the spec does not
    /// conform; instead, this field is called `text`.
    #[serde(rename = "text")]
    pub description: String,
    /// This attribute contains the ISO code for the language of the
    /// description. Alternately it may be null to indicate the default
    /// language, i.e. English.
    #[serde(default)]
    pub lang: Option<Lang>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DescriptionRef {
    /// This attribute contains the reference.
    pub refid: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Documentation {
    /// This attribute contains the ISO code for the language of the
    /// description.
    pub lang: Lang,

    /// This attribute contains the (English) text describing this documentation
    /// item.
    pub details: String,

    /// This attribute contains a reference to the documentation. The prefix
    /// ctan: indicates a reference to a directory on CTAN. If the parameter
    /// keep-url is true then this attribute contains always a valid URL without
    /// the ctan: prefix.
    pub href: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CtanLocation {
    /// This attribute contains the relative path of the package in the CTAN
    /// tree. This attribute is mandatory.
    pub path: RelativePath,

    /// This attribute contains the indicator that this package consists of a
    /// single file only. This is in contrast to a whole package directory. This
    /// attribute is optional and defaults to false.
    #[serde(default)]
    pub file: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Install {
    /// This attribute contains path relative to the CTAN directory /install.
    /// This attribute is mandatory.
    pub path: RelativePath,
}

/// NOTE: According to the spec, this is supposed to contain a field. However,
/// it's simply a string for the example package (`tex`) that the spec talks about.
#[derive(Debug, Clone, Deserialize)]
#[serde(from = "String")]
pub struct Miktex {
    /// This attribute contains name of the package in MiKTEX. This attribute is mandatory.
    pub location: String,
}

impl From<String> for Miktex {
    fn from(location: String) -> Self {
        Self { location }
    }
}

/// NOTE: According to the spec, this is supposed to contain a field. However,
/// it's simply a string for the example package (`tex`) that the spec talks about.
#[derive(Debug, Clone, Deserialize)]
#[serde(from = "String")]
pub struct Texlive {
    /// This attribute contains name of the package in TEX Live. This attribute is mandatory.
    pub location: String,
}

impl From<String> for Texlive {
    fn from(location: String) -> Self {
        Self { location }
    }
}

#[cfg(test)]
mod tests {
    use crate::conf::{DependencyName, DependencyVersion};

    use super::super::WebClient;

    #[tokio::test]
    async fn get_pkg_metadata_works() {
        let client = WebClient::new().unwrap();
        // FIXME: This is a pretty awkward way to construct one of these things!
        let name: crate::conf::DependencyName<'static> = unsafe { std::mem::transmute("tex") };
        let pkg = client
            .get_ctan_pkg_metadata(&name, &DependencyVersion::Any)
            .await
            .unwrap();
        assert_eq!(&pkg.authors[0].id, "knuth");
    }
}
