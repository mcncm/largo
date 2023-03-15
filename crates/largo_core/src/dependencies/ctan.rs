#![allow(unused)]

use serde::Deserialize;

pub type Year = i32;

pub type Lang<'p> = &'p str;

pub type RelativePath<'p> = &'p str;

pub type PackageId<'p> = &'p str;

/// A package from the CTAN database as queried with the JSON api and specified
/// at [this](https://ctan.org/help/json/2.0/pkg) page.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Package<'p> {
    /// This attribute contains the unique id of the package. This attribute is
    /// mandatory.
    id: PackageId<'p>,

    /// This attribute contains a list of aliases for the package. The alias is
    /// a object which has several attributes:
    #[serde(borrow, default)]
    aliases: Vec<Alias<'p>>,

    /// The entry has the mandatory attribute <name>. The name contains the
    /// print representation of the package name.
    name: &'p str,

    /// The entry has the mandatory attribute <caption>. The caption contains a
    /// short description of the package.
    caption: &'p str,

    /// The entry has the attribute authors which contains a list of authors.
    /// The author is a object which has several attributes:
    #[serde(borrow, default)]
    authors: Vec<Author<'p>>,

    /// The entry can have a list-valued attribute copyright. It carries the
    /// information about the copyright. This list contains objects which have
    /// several attributes:
    #[serde(borrow, default)]
    copyright: Vec<Copyright<'p>>,

    /// The entry can have an attribute license.
    #[serde(borrow)]
    license: License<'p>,

    /// The entry has the attribute version. It carries the information about
    /// the version of the package. This object has several attributes.
    #[serde(borrow)]
    version: Version<'p>,

    /// The entry has a list of description objects. It may have attributes:
    #[serde(borrow, default)]
    descriptions: Vec<Description<'p>>,

    /// An inner tag of <description> is <ref>. It is used to reference a
    /// package. The tag may have an attribute:
    #[serde(borrow, rename = "<ref>")]
    r#ref: DescriptionRef<'p>,

    /// The entry has the list attribute documentation. The list elements
    /// indicate references to documentation. The objects may have attributes:
    #[serde(borrow, default)]
    documentation: Vec<Documentation<'p>>,

    /// The entry has the optional attribute ctan. It carries the location of
    /// the package in the CTAN tree. This JSON object has several attributes:
    #[serde(borrow, default)]
    ctan: Option<CtanLocation<'p>>,

    /// The entry has the optional attribute install. It carries the location of
    /// the package on CTAN in form of an installable TDS-compliant zip archive.
    /// This JSON object has several attributes.
    #[serde(borrow, default)]
    install: Option<Install<'p>>,

    /// The entry has the optional attribute miktex. It carries the name of the
    /// package in MiKTEX. This JSON object has several attributes.
    #[serde(borrow, default)]
    miktex: Option<Miktex<'p>>,

    /// The entry has the optional attribute texlive. It carries the name of the
    /// package in TEX Live. This JSON object has several attributes:
    #[serde(borrow, default)]
    texlive: Option<Texlive<'p>>,

    /// The entry has the optional attribute index. If present then it contains
    /// a list of extra terms to be indexed for the search.
    #[serde(borrow, default)]
    index: Option<Vec<&'p str>>,

    /// The entry has the optional attribute topics. If present then it contains
    /// a list of topics keys for this entry.
    #[serde(borrow, default)]
    topics: Option<Vec<&'p str>>,

    /// The entry has the optional attribute home. If present then it contains
    /// the URL of the home page of the package.
    #[serde(borrow, default)]
    home: Option<&'p str>,

    /// The entry has the optional attribute support. If present then it
    /// contains the URL of the support for the package.
    #[serde(borrow, default)]
    support: Option<&'p str>,

    /// The entry has the optional attribute announce. If present then it
    /// contains the URL of the announcements for the package.
    #[serde(borrow, default)]
    announce: Option<&'p str>,

    /// The entry has the optional attribute bugs. If present then it contains
    /// the URL of the bug tracker for the package.
    #[serde(borrow, default)]
    bugs: Option<&'p str>,

    /// The entry has the optional attribute repository. If present then it
    /// contains the URL of the source code repository for the package.
    #[serde(borrow, default)]
    repository: Option<&'p str>,

    /// The entry has the optional attribute development. If present then it
    /// contains the URL of the developer community for the package.
    #[serde(borrow, default)]
    development: Option<&'p str>,
}

#[derive(Debug, Clone, Deserialize)]
struct Alias<'p> {
    /// This attribute contains the id of the alias. This attribute is mandatory.
    id: PackageId<'p>,
    /// This attribute is the name of the alias. It is mandatory.
    name: &'p str,
}

pub type AuthorId<'p> = &'p str;

#[derive(Debug, Clone, Deserialize)]
struct Author<'p> {
    /// This attribute contains the id of the author. This attribute is
    /// mandatory.
    id: AuthorId<'p>,

    /// This attribute is the title of the author. It is optional and can be
    /// empty. The default is empty.
    #[serde(default)]
    title: Option<&'p str>,

    /// This attribute contains the given name. It is optional and can be empty.
    #[serde(default)]
    givenname: Option<&'p str>,

    /// This attribute is the von part of the author's name. It is usually in
    /// lower case and has values like von, van, or de. It is optional and can
    /// be empty. The default is empty.
    #[serde(default)]
    von: Option<&'p str>,

    /// This attribute is the family name. It is optional and can be empty.
    #[serde(default)]
    familyname: Option<&'p str>,

    /// This attribute is the junior part of the author's name. It is usually an
    /// addition to the name like jr., sr., or a numeral like I, II, III, IV. It
    /// is optional and can be empty. The default is empty.
    #[serde(default)]
    junior: Option<&'p str>,

    /// This attribute is the alias name to protect the privacy of an author who
    /// requests it. It is optional and can be empty. The default is empty. In
    /// case this attribute is not empty the other name constituents are not
    /// shown.
    #[serde(default)]
    pseudonym: Option<&'p str>,

    /// This attribute is the boolean indicator that the author is female. It is
    /// optional and can be empty. The default is false.
    ///
    /// NOTE: I didn't write this spec!!!
    #[serde(default)]
    female: bool,

    /// This attribute is the indicator that the author is deceased. It is
    /// optional and can be empty. The default is empty.
    #[serde(default)]
    died: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Copyright<'p> {
    /// This attribute contains the name of the copyright holder. This attribute
    /// is mandatory.
    owner: &'p str,

    /// This attribute contains the year or years of the copyright. This
    /// attribute is mandatory.
    year: Year,
}

/// At least one of number or date have to be given. Otherwise the tag is
/// suppressed.
#[derive(Debug, Clone, Deserialize)]
pub struct Version<'p> {
    /// This attribute contains the version number.
    #[serde(default)]
    number: Option<&'p str>,
    /// This attribute contains the version date.
    #[serde(default)]
    date: Option<&'p str>,
}

/// The value can be either a string or a list of strings with keys of licenses.
#[derive(Debug, Clone, Deserialize)]
pub enum License<'p> {
    String(&'p str),
    /// FIXME: I don't really know what they want here.
    List(Vec<&'p str>),
}

#[derive(Debug, Clone, Deserialize)]
pub struct Description<'p> {
    /// This attribute contains the longer description of the package. It may include HTML markup.
    description: &'p str,
    /// This attribute contains the ISO code for the language of the
    /// description. Alternately it may be null to indicate the default
    /// language, i.e. English.
    #[serde(default)]
    lang: Option<Lang<'p>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DescriptionRef<'p> {
    /// This attribute contains the reference.
    refid: &'p str,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Documentation<'p> {
    /// This attribute contains the ISO code for the language of the
    /// description.
    lang: Lang<'p>,

    /// This attribute contains the (English) text describing this documentation
    /// item.
    details: &'p str,

    /// This attribute contains a reference to the documentation. The prefix
    /// ctan: indicates a reference to a directory on CTAN. If the parameter
    /// keep-url is true then this attribute contains always a valid URL without
    /// the ctan: prefix.
    href: &'p str,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CtanLocation<'p> {
    /// This attribute contains the relative path of the package in the CTAN
    /// tree. This attribute is mandatory.
    #[serde(borrow)]
    path: RelativePath<'p>,

    /// This attribute contains the indicator that this package consists of a
    /// single file only. This is in contrast to a whole package directory. This
    /// attribute is optional and defaults to false.
    #[serde(default)]
    file: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Install<'p> {
    /// This attribute contains path relative to the CTAN directory /install.
    /// This attribute is mandatory.
    #[serde(borrow)]
    path: RelativePath<'p>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Miktex<'p> {
    /// This attribute contains name of the package in MiKTEX. This attribute is mandatory.
    location: &'p str,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Texlive<'p> {
    /// This attribute contains name of the package in TEX Live. This attribute is mandatory.
    location: &'p str,
}
