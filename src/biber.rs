#![allow(unused)]

type AnnotationMarker = String;

type CollateOptions = String;

// TODO: not sure about this type
type MinCrossrefs = i32;

enum DecodeCharSet {
    Full,
    Base,
}

enum DotIncludeElement {
    Section,
    Field,
    Xdata,
    Crossref,
    Xref,
    Related,
}

enum Encoding {
    Utf8,
}

enum InputFormat {
    Bibtex,
    Biblatexml,
}

type NamedAnnotationMarker = String;

enum OutputFieldcase {
    Upper,
    Lower,
    Title,
}

enum OutputFormat {
    Dot,
    Bibtex,
    Biblatexml,
    Bbl,
    Bblxml,
}

enum OutputIndent {
    Spaces(u8),
    Tabs(u8),
}

enum OutputSafecharset {
    Full,
    Base,
    Null,
}

type Locale = String;

// #[derive(clam::Options)]
// #[clam(case_convention = "two_dash_kebab_case")]
struct CommandLineOptions {
    /// Sets the suffix which can be appended to a BibTeX data source field
    /// name to indicate that the value of the field is a data annotation.
    /// The default is "+an".
    annotation_marker: Option<AnnotationMarker>,

    /// If running as a PAR::Packer binary, show the cache location and
    /// exit.
    cache: bool,

    /// Clears any BibTeX macros (@STRING) between BibLaTeX refsections.
    /// This prevents BibTeX warnings about macro redefinitions if you are
    /// using the same datasource several times for different refsections.
    clrmacros: bool,

    /// Options to pass to the "Unicode::Collate" object used for sorting
    /// (default is 'level => "4", variable => "non-ignorable"'). See
    /// "perldoc Unicode::Collate" for details.
    collate_options: Option<CollateOptions>,

    /// Use file as the configuration file for "biber" instead of looking in
    /// the default locations which are, in order:
    ///
    /// * "biber.conf" or ".biber.conf" in the current directory
    ///
    /// * "$HOME/.biber.conf"
    ///
    /// * "$ENV{XDG_CONFIG_HOME}/biber/biber.conf"
    ///
    /// * "$HOME/.config/biber/biber.conf"
    ///
    /// * "$HOME/Library/biber/biber.conf" (Mac OSX only)
    ///
    /// * "$ENV{APPDATA}/biber.conf" (Windows only)
    ///
    /// * The output of "kpsewhich biber.conf" (if available on the system).
    ///
    /// In tool mode, (--tool) the biber-tool.conf installed with Biber is
    /// always used to set default options if a user-defined config file is
    /// not specified. Use the --tool-config option to view the location of
    /// the default tool mode config file which may be useful as a source
    /// for writing your own.
    configfile: Option<std::path::PathBuf>,

    /// Converts the .bcf control file into html using an XSLT transform.
    /// Can be useful for debugging. File is named by appending ".html" to
    /// .bcf file.
    convert_control: bool,

    /// The set of characters included in the conversion routine when
    /// decoding LaTeX macros into UTF-8 (which happens when
    /// --bblencoding|-E is set to UTF-8). Set to "full" to try harder with
    /// a much larger set or "base" to use a smaller basic set. Default is
    /// "base". You may want to try "full" if you have less common UTF-8
    /// characters in your data source. The recode sets are defined in the
    /// reencoding data file which can be customised. See the --recodedata
    /// option and the PDF manual. The virtual set name "null" may be
    /// specified which effectively turns off macro decoding.
    decodecharsset: Option<DecodeCharSet>,

    /// Turn on debugging for "biber".
    debug: bool,

    /// Exit immediately with error if using "--validate-datamodel" and a
    /// datamodel validation error is found. Default is to warn and
    /// continue.
    dieondatamodel: bool,

    /// Specifies the element to include in GraphViz DOT output format if
    /// the output format is 'dot'. You can also choose to display crossref,
    /// xref, xdata and/or related entry connections. The default if not
    /// specified is "--dot-include=section,xdata,crossref,xref".
    dot_include: Option<std::collections::HashSet<DotIncludeElement>>,

    /// Try to fix broken multiple initials when they have no space between
    /// them in BibTeX data sources. That is, "A.B. Clarke" becomes "A. B.
    /// Clarke" before name parsing. This can slightly mess up things like
    /// "{U.K. Government}" and other esoteric cases.
    fixinits: bool,

    /// Show this help message.
    help: bool,

    /// .bcf and data files will be looked for first in the directory. See
    /// the biber PDF documentation for the other possibilities and how this
    /// interacts with the "--output-directory" option.
    input_directory: Option<std::path::PathBuf>,

    /// Specify the encoding of the data source file(s). Default is "UTF-8"
    /// Normally it's not necessary to set this as it's passed via the .bcf
    /// file from biblatex's "bibencoding" option. See "perldoc
    /// Encode::Supported" for a list of supported encodings. The legacy
    /// option --bibencoding is supported as an alias.
    input_encoding: Option<Encoding>,

    /// Biber input format. This option only means something in tool mode
    /// (see tool option) since normally the input format of a data source
    /// is specified in the .bcf file and therefore from the \addbibresouce
    /// macro in BibLaTeX. The default value when in tool mode is 'bibtex'
    input_format: Option<InputFormat>,

    /// Force all ISBNs to 10-digit versions on output. This will convert
    /// the ISBN internally to an ISBN object which will not have hyphens on
    /// output. If you use this option and want an ISBN with hyphens in the
    /// correct place on output, use the --isbn-normalise option.
    isbn10: bool,

    /// Force all ISBNs to 13-digit versions on output. This will convert
    /// the ISBN internally to an ISBN object which will not have hyphens on
    /// output. If you use this option and want an ISBN with hyphens in the
    /// correct place on output, use the --isbn-normalise option.
    isbn13: bool,

    /// Normalise ISBNs with hyphens in the correct places on output.
    isbn_normalise: bool,

    /// Use file.blg as the name of the logfile.
    logfile: Option<String>,

    /// Use sep as the separator for BibTeX data source list fields.
    /// Defaults to BibTeX's usual 'and'.
    listsep: Option<String>,

    /// Set threshold for crossrefs.
    mincrossrefs: Option<MinCrossrefs>,

    /// Sets the separator between the "--annotation-marker" and the name of
    /// a named annotation. The default is ":".
    named_annotation_marker: Option<NamedAnnotationMarker>,

    /// Use sep as the separator for BibTeX data source name fields.
    /// Defaults to BibTeX's usual 'and'.
    namesep: Option<String>,

    /// When writing bblxml output, don't generate an RNG XML schema from
    /// the data model.
    no_bblxml_schema: bool,

    /// When reading or writing biblatexml data sources, don't generate an
    /// RNG XML schema from the data model.
    no_bltxml_schema: bool,

    /// Don't look for a configfile.
    noconf: bool,

    /// Do not load the default datamodel coming from either the .bcf or,
    /// when in tool mode, from the default tool mode config file. Use of
    /// this option implies that you will provide a complete datamodel in a
    /// config file. This option is useful when you wish to make major
    /// modifications to the datamodel because the simple add/modify
    /// operations to the default datamodel via a user config file are not
    /// enough. For example, to remove things from the default datamodel,
    /// you would use this option and supply a complete, reduced datamodel
    /// in the user config file.
    no_default_datamodel: bool,

    /// Don't exit on errors, just log and continue as far as possible. This
    /// can be useful if the error is something from, for example, the
    /// underlying BibTeX parsing C library which can complain about parsing
    /// errors which can be ignored.
    nodieonerror: bool,

    /// By default, glob (expand according to pattern) any data source
    /// filenames. Allows data sources to be specified like *.bib to load
    /// all .bib files in a directory. Can be overridden on a per-dataource
    /// basis with the glob option to \addbibresource in biblatex.
    glob_datasources: bool,

    /// Do not write any logfile.
    nolog: bool,

    /// Don't skip duplicate bibliography keys if found. The detection of
    /// duplicate keys is done across all data sources. Sometimes you might
    /// need duplicates when using several data sources across several
    /// refsections in which case you might need to use this option.
    noskipduplicates: bool,

    /// Don't automatically define any standard macros like month
    /// abbreviations. If you also define these yourself, this option can be
    /// used to suppress macro redefinition warnings.
    nostdmacros: bool,

    /// Do not remove the temporary directory used for various intermediate
    /// files and data before exit (default is false). Name of the directory
    /// can be obtained with the --show-tmp-dir option.
    noremove_tmp_dir: bool,

    /// Disable exended name processing in bibtex data sources. Can be
    /// useful if you don't use this and it causes problems due to
    /// auto-detection of extended name format.
    noxname: bool,

    /// Do not write any message to screen.
    onlylog: bool,

    /// Use string as the final name in a name field which implies "et al".
    /// Defaults to BibTeX's usual 'others'.
    others_string: Option<String>,

    /// Align field values in neat columns in output. Effect depends on the
    /// output format. Default is false. The legacy option --tool-align is
    /// supported as an alias.
    output_align: bool,

    /// When outputting bibtex format, whether to output all found macro
    /// (@STRING entries) definitions rather than just definitions for
    /// macros which are actually used in the output entries. Default is
    /// false.
    output_all_macrodefs: bool,

    /// As --annotation-marker but for tool mode bibtex output. The default
    /// is "+an".
    output_annotation_marker: Option<AnnotationMarker>,

    /// As --named-annotation-marker but for tool mode bibtex output. The
    /// default is ":".
    output_named_annotation_marker: Option<NamedAnnotationMarker>,

    /// Output files (including log files) are output to directory instead
    /// of the current directory. Input files are also looked for in
    /// directory before current directory unless "--input-directory" is
    /// also specified in which case input files are only looked for in the
    /// directory specified by "--input-directory".
    output_directory: Option<std::path::PathBuf>,

    /// Specify the encoding of the output ".bbl" file. Default is "UTF-8".
    /// Normally it's not necessary to set this as it's passed via the .bcf
    /// file from biblatex's "texencoding" option. See "perldoc
    /// Encode::Supported" for a list of supported encodings. The legacy
    /// option --bblencoding is supported as an alias.
    output_encoding: Option<Encoding>,

    /// Case for field names output. Effect depends on the output format.
    /// Defaults to 'upper'. The legacy option --tool-fieldcase is supported
    /// as an alias.
    output_fieldcase: Option<OutputFieldcase>,

    /// When outputting bibtex format data in tool mode, this option allows
    /// the customisation of the order of fields within entries. The value
    /// is a comma-separated string of field names or classes of fields.
    /// Fields not mentioned in the list are output in sorted name order
    /// after the explicitly specified fields. The classes of fields are:
    ///
    /// 'names' - All name list fields 'lists' - All non-name list fields
    /// 'dates' - All date fields
    ///
    /// By default, its value is 'options,abstract,names,lists,dates'.
    output_field_order: Option<Vec<String>>,

    /// When outputting bibtex format output "replacefieldn" instead of
    /// "fieldn". This can be used to output legacy formats which undo the
    /// default driver source map e.g
    /// --output-field-replace=location:address,journaltitle:journal. See
    /// --output-legacy-dates if legacy (YEAR/MONTH) date fields are
    /// required in bibtex format output.
    output_field_replace: Option<std::collections::HashMap<String, String>>,

    /// Output to file instead of basename.bbl file is relative to
    /// --output-directory, if set (absolute paths in this case are stripped
    /// to filename only). file can be absolute if --output-directory is not
    /// set. file can be '-' to output directly to STDOUT. The legacy option
    /// --outfile is supported as an alias.
    output_file: Option<std::path::PathBuf>,

    /// Biber output format. Default if not specified is of course, bbl. Use
    /// dot to output a GraphViz DOT file instead of .bbl. This is a
    /// directed graph of the bibliography data showing entries and, as
    /// requested, sections and fields. You must process this file with
    /// "dot", e.g. "dot -Tsvg test.dot -o test.svg" to render the graph.
    /// See the --dot-include option to select what is included in the DOT
    /// output. bblxml is an XML version of the bbl format which you could
    /// transform using XSLT. By default, when outputting bblxml, a RelaxNG
    /// XML schema is generated from the active data model and saved with a
    /// rng extension along with the output file name (unless the
    /// --no-bblxml-schema option is specified). You may validate the bblxml
    /// using the schema with the --validate-bblxml option. The legacy
    /// option --outformat is supported as an alias.
    output_format: Option<OutputFormat>,

    /// Indentation for body of entries in output. Effect depends on the
    /// output format. Defaults to 2. The "num" can be followed by 't' to
    /// specify tabs instead of spaces. The legacy option --tool-indent is
    /// supported as an alias.
    output_indent: Option<OutputIndent>,

    /// When outputting bibtex format, output YEAR/MONTH fields instead of
    /// DATE. This is not possible if the input is not convertible to legacy
    /// format, meaning that any date to be output with legacy fields can
    /// only have a YEAR part and perhaps a MONTH part. If a DAY or ENDYEAR
    /// part are found, the date is not convertible and the legacy output
    /// format will be skipped. Default is false.
    output_legacy_dates: bool,

    /// As --listsep but for tool mode bibtex output. Defaults to BibTeX's
    /// usual 'and'.
    output_listsep: Option<String>,

    /// As --namesep but for tool mode bibtex output. Defaults to BibTeX's
    /// usual 'and'.
    output_namesep: Option<String>,

    /// When outputting BibTeX format, don't output macro definitions
    /// (@STRING entries). You might not want to output macro definitions if
    /// you keep them in a separate file.
    output_no_macrodefs: bool,

    /// Convenience option to set all of the --output-resolve-* options to
    /// 'true'. The legacy option --tool-resolve is supported as an alias.
    output_resolve: bool,

    /// Whether to resolve XDATA inheritance in tool mode or when
    /// --output-format=bibtex in non tool mode. Defaults to 'false'.
    output_resolve_xdata: bool,

    /// Whether to resolve CROSSREF/XREF inheritance in tool mode or when
    /// --output-format=bibtex in non tool mode. Defaults to 'false'.
    output_resolve_crossrefs: bool,

    /// Whether to resolve data sets in tool mode or when
    /// --output-format=bibtex in non tool mode. Defaults to 'false'.
    output_resolve_sets: bool,

    /// Try to convert UTF-8 chars into LaTeX macros when writing the
    /// output. This can prevent unknown char errors when using PDFLaTeX and
    /// inputenc as this doesn't understand all of UTF-8. Note, it is better
    /// to switch to XeTeX or LuaTeX to avoid this situation. By default
    /// uses the --output-safecharsset "base" set of characters. The legacy
    /// option --bblsafechars is supported as an alias.
    output_safechars: bool,

    /// The set of characters included in the conversion routine for
    /// --output-safechars. Set to "full" to try harder with a much larger
    /// set or "base" to use a basic set. Default is "base" which is fine
    /// for most use cases. You may need to load more macro packages to deal
    /// with the results of "full" (Dings, Greek characters, special symbols
    /// etc.). The recode sets are defined in the reencoding data file which
    /// can be customised. See the --recodedata option and the PDF manual.
    /// The legacy option --bblsafecharsset is supported as an alias. The
    /// virtual set name "null" may be specified which effectively turns off
    /// macro encoding.
    output_safecharsset: Option<OutputSafecharset>,

    /// As --xdatamarker but for tool mode output. Default is 'xdata'.
    output_xdatamarker: Option<String>,

    /// As --xdatasep but for tool mode output. Default is '-'.
    output_xdatasep: Option<String>,

    /// When output is a .bib BibTeX file in tool mode, whether to output
    /// names the eXtended BibTeX name field format.
    output_xname: bool,

    /// As --xnamesep but for tool mode bibtex output. Default is '='.
    output_xnamesep: Option<String>,

    /// Log only errors. If this option is used more than once, don't even
    /// log errors.
    quiet: bool,

    /// The data file to use for the reencoding between UTF-8 and LaTeX
    /// macros. It defines the sets specified with the --output-safecharsset
    /// and --decodecharsset options. It defaults to recode_data.xml in the
    /// same directory as Biber's Recode.pm module. See the PDF
    /// documentation for the format of this file. If this option is used,
    /// then file should be somewhere "kpsewhich" can find it.
    recodedata: Option<std::path::PathBuf>,

    /// Prints the location of the temporary directory used for various
    /// intermediate files and data. Only useful if --noremove-tmp-dir is
    /// set to true.
    show_tmp_dir: bool,

    /// Add comments to output with sorting keys. Useful for debugging.
    sortdebug: bool,

    /// Case-sensitive sorting (default is true).
    sortcase: Option<bool>,

    /// Set the locale to be used for sorting. The locale is used to add
    /// CLDR tailoring to the sort (if available for the locale).
    sortlocale: Locale,

    /// Whether to sort uppercase before lowercase when sorting (default is
    /// true).
    sortupper: Option<bool>,

    /// Don't try to use the default Mozilla CA certificates when using
    /// HTTPS to fetch remote data. This assumes that the user will set one
    /// of the perl LWP::UserAgent module environment variables to find the
    /// CA certs.
    ssl_nointernalca: bool,

    /// Turn off host verification when using HTTPS to fetch remote data
    /// sources. You may need this if the SSL certificate is self-signed for
    /// example.
    ssl_noverify_host: bool,

    /// In tool mode, strip all comments from the output file.
    strip_comments: bool,

    /// Run in tool mode. This mode is datasource centric rather than
    /// document centric. biber reads a datasource (and a config file if
    /// specified), applies the command-line and config file options to the
    /// datasource and writes a new datasource. Essentially, this allows you
    /// to change your data sources using biber's transformation options
    /// (such as source mapping, sorting etc.)
    tool: bool,

    /// Show the location of the default tool mode config file and exit.
    /// Useful when you need to copy this file and customise it.
    tool_config: bool,

    /// Use this option in tool mode if you don't want to remove "XREF",
    /// "CROSSREF" and "XDATA" fields from the output which point to a
    /// missing entry. Useful if you have split datafiles where the e.g.
    /// "CROSSREF"s are in another file that you are not including in the
    /// tool mode run.
    tool_noremove_missing_dependants: bool,

    /// Turn on tracing. Also turns on --debug|d and additionally provides a
    /// lot of low-level tracing information in the log.
    trace: bool,

    // -u  Alias for --input-encoding=UTF-8

    // -U  Alias for --output-encoding=UTF-8
    /// Schema validate bblXML output against a schema auto-generated from
    /// the BibLaTeX datamodel. The schema will be auto-generated with the
    /// name of the .bbl file with a .rng extension. The generated schema
    /// can be kept and used with standard XML editors to validate the
    /// output during XSL development.
    validate_bblxml: bool,

    /// Schema validate BibLaTeXML datasources against a schema
    /// auto-generated from the BibLaTeX datamodel. The schema will be
    /// auto-generated with the name of the .bcf file with a .rng extension.
    /// The generated schema can be kept and used with standard XML editors
    /// to validate the datasource during datasource development. The schema
    /// validation does not validate all semantic aspects of the datamodel
    /// (i.e. the data model constraints)---for this use the
    /// "--validate-datamodel" option.
    validate_bltxml: bool,

    /// Schema validate the biber config file.
    validate_config: bool,

    /// Schema validate the .bcf biblatex control file.
    validate_control: bool,

    /// Validate the data against a data model.
    validate_datamodel: bool,

    /// Display version number.
    version: bool,

    /// In Windows 10 1803+, turning on the 'Use Unicode UTF8 for worldwide
    /// language support' option makes Windows use UTF-8 instead of UTF-16
    /// for many system APIs. If that option is enabled, use this option to
    /// turn off biber's UTF-16 filename support on Windows. This will
    /// result in much improved handling of Unicode filenames.
    winunicode: bool,

    /// Wrap lines in the .bbl file.
    wraplines: bool,

    /// Use marker as the string before "--xdatasep" which introduces an
    /// XDATA reference in BibTeX format data sources. Not used in
    /// BibLaTeXML data sources as it has a dedicated XML attribute "xdata"'
    /// for this. Default is 'xdata'.
    xdatamarker: Option<String>,

    /// Use sep as the separator between XDATA sub-entry parts in the
    /// eXtended name format. See biber docs. Default is '-'.
    xdatasep: Option<String>,

    /// Use sep as the separator between namepart names and the namepart
    /// values in the eXtended name format. Also applies to XDATA references
    /// as the separator between "--xdatamarker" and the XDATA reference.
    /// See biber docs. Default is '='.
    xnamesep: Option<String>,

    /// Use sep as the separator for fields of format type "xsv" in the data
    /// model. A Perl regexp can be specified. Defaults to a single comma
    /// surround by optional whitespace (\s*,\s*).
    xsvsep: Option<String>,
}
