use serde::Serialize;

pub struct Pdflatex<'a> {
    _binary: &'a dyn AsRef<std::ffi::OsStr>,
    _cli_options: CommandLineOptions,
}

impl<'a> super::TexEngine for Pdflatex<'a> {}

#[derive(Debug, Clone, Serialize)]
pub enum InteractionMode {
    BatchMode,
    NonStopMode,
    ScrollMode,
    ErrorStopMode,
}

impl clam::ArgValue for InteractionMode {
    fn set_cmd_arg(&self, name: &str, cmd: &mut std::process::Command) {
        let mode = match self {
            InteractionMode::BatchMode => "batchmode",
            InteractionMode::NonStopMode => "nonstopmode",
            InteractionMode::ScrollMode => "scrollmode",
            InteractionMode::ErrorStopMode => "errorstopmode",
        };
        cmd.args([name, &mode]);
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum MkTexFormat {
    Tex,
    Tfm,
    Pk,
}

impl clam::ArgValue for MkTexFormat {
    fn set_cmd_arg(&self, name: &str, cmd: &mut std::process::Command) {
        let format = match self {
            MkTexFormat::Tex => "tex",
            MkTexFormat::Tfm => "tfm",
            MkTexFormat::Pk => "pk",
        };
        cmd.args([name, &format]);
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum SrcSpecial {
    Cr,
    Display,
    Hbox,
    Math,
    Par,
    Parend,
    Vbox,
}

impl clam::ArgValue for SrcSpecial {
    fn set_cmd_arg(&self, name: &str, cmd: &mut std::process::Command) {
        let special = match self {
            SrcSpecial::Cr => "cr",
            SrcSpecial::Display => "display",
            SrcSpecial::Hbox => "hbox",
            SrcSpecial::Math => "math",
            SrcSpecial::Par => "par",
            SrcSpecial::Parend => "parend",
            SrcSpecial::Vbox => "vbox",
        };
        cmd.args([name, &special]);
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum Format {
    Pdf,
    Dvi,
}

impl clam::ArgValue for Format {
    fn set_cmd_arg(&self, name: &str, cmd: &mut std::process::Command) {
        let format = match self {
            Format::Pdf => "pdf",
            Format::Dvi => "dvi",
        };
        cmd.args([name, &format]);
    }
}

pub type ConfigurationFileLine = String;

pub type TcxName = String;

/// Syntex option type
pub type SynctexNumber = i32;

/// Kpathsea debug option type
pub type KpathseaNumber = i32;

#[allow(dead_code)]
#[derive(Debug, Default, clam::Options)]
#[clam(case_convention = "one_dash_kebab_case")]
pub struct CommandLineOptions {
    /// parse STRING as a configuration file line
    cnf_line: Option<ConfigurationFileLine>,
    /// switch on draft mode (generates no output PDF)
    draftmode: bool,
    /// enable encTeX extensions such as \mubyte
    enc: bool,
    /// enable e-TeX extensions
    etex: bool,
    /// enable file:line:error style messages
    file_line_error: bool,
    /// disable file:line:error style messages
    no_file_line_error: bool,
    /// use FMTNAME instead of program name or a %& line
    fmt: Option<String>,
    /// stop processing at the first error
    halt_on_error: bool,
    /// be pdfinitex, for dumping formats; this is implicitly true if the program name is `pdfinitex'
    ini: bool,
    /// set interaction mode (STRING=batchmode/nonstopmode/scrollmode/errorstopmode)
    pub interaction: Option<InteractionMode>,
    /// send DVI output to a socket as well as the usual output file
    ipc: bool,
    /// as -ipc, and also start the server at the other end
    ipc_start: bool,
    /// set the job name to STRING
    jobname: Option<String>,
    /// set path searching debugging flags according to the bits of NUMBER
    kpathsea_debug: Option<KpathseaNumber>,
    /// enable mktexFMT generation (FMT=tex/tfm/pk)
    mktex: Option<MkTexFormat>,
    /// disable mktexFMT generation (FMT=tex/tfm/pk)
    no_mktex: Option<MkTexFormat>,
    /// enable MLTeX extensions such as \charsubdef
    mltex: bool,
    /// use STRING for DVI file comment instead of date (no effect for PDF)
    output_comment: Option<String>,
    /// use existing DIR as the directory to write files in
    output_directory: Option<std::path::PathBuf>,
    /// use FORMAT for job output; FORMAT is `dvi' or `pdf'
    output_format: Option<Format>,
    /// enable parsing of first line of input file
    parse_first_line: bool,
    /// disable parsing of first line of input file
    no_parse_first_line: bool,
    /// set program (and fmt) name to STRING
    progname: Option<String>,
    /// enable filename recorder
    recorder: bool,
    /// enable \write18{SHELL COMMAND}
    pub shell_escape: bool,
    /// disable \write18{SHELL COMMAND}
    pub no_shell_escape: bool,
    /// enable restricted \write18
    shell_restricted: bool,
    /// insert source specials in certain places of the DVI file. WHERE is a comma-separated value list: cr display hbox math par parend vbox
    // We interpret `Option<Vec![]>` as `-src-specials` without the list
    src_specials: Option<Vec<SrcSpecial>>,
    /// generate SyncTeX data for previewers according to bits of NUMBER (`man synctex' for details)
    synctex: Option<SynctexNumber>,
    /// use the TCX file TCXNAME
    translate_file: Option<TcxName>,
    // FIXME: rename to `8bit`
    /// make all characters printable by default
    eight_bit: bool,
    /// display this help and exit
    help: bool,
    /// output version information and exit
    version: bool,
}
