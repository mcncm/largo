use serde::Serialize;

use super::{private::CommandBuilder, Engine, EngineBuilder};
use crate::{dirs, Result};

pub struct PdflatexBuilder {
    cmd: crate::Command,
    texinputs: Vec<String>,
    cli_options: CommandLineOptions,
}

impl CommandBuilder for PdflatexBuilder {
    fn inner_cmd(&self) -> &crate::Command {
        &self.cmd
    }

    fn inner_cmd_mut(&mut self) -> &mut crate::Command {
        &mut self.cmd
    }
}

impl PdflatexBuilder {
    // NOTE: Only using `conf` just to find its own executable. In fact, it
    // should probably be using some _other_ input; that's more data than it
    // should have access to.
    pub fn new(conf: &crate::conf::LargoConfig) -> Self {
        let cmd = crate::Command::new(&conf.build.execs.pdflatex);
        let cli_options = CommandLineOptions {
            // Always use nonstop mode for now.
            interaction: Some(InteractionMode::NonStopMode),
            ..Default::default()
        };
        Self {
            cmd,
            cli_options,
            texinputs: Vec::new(),
        }
    }

    fn disable_line_wrapping(&mut self) {
        // FIXME: you should be able to do this as a static converstion to a
        // &'static str, and without an allocation.
        self.cmd.env("max_print_line", &i32::MAX.to_string());
    }
}

impl EngineBuilder for PdflatexBuilder {
    fn with_src_dir<P: typedir::AsPath<dirs::SrcDir>>(mut self, path: P) -> Self {
        // FIXME: unnecessary allocation
        self.texinputs.push(format!("{}", path.as_ref().display()));
        self
    }

    fn with_verbosity(self, _verbosity: &crate::build::Verbosity) -> Self {
        // FIXME: just a no-op for now
        self
    }

    fn with_synctex(mut self, use_synctex: bool) -> Result<Self> {
        if use_synctex {
            self.cli_options.synctex = Some(SYNCTEX_GZIPPED);
        }
        Ok(self)
    }

    fn with_jobname(mut self, jobname: String) -> Result<Self> {
        self.cli_options.jobname = Some(jobname);
        Ok(self)
    }

    fn with_shell_escape(mut self, shell_escape: Option<bool>) -> Result<Self> {
        match shell_escape {
            Some(true) => {
                self.cli_options.shell_escape = true;
            }
            Some(false) => {
                self.cli_options.no_shell_escape = true;
            }
            None => (),
        }
        Ok(self)
    }

    fn finish(mut self) -> Engine {
        // Appy environment variables
        self.disable_line_wrapping();
        let mut cmd = self.cmd;
        let mut texinputs = self.texinputs.join(":");
        texinputs += ":";
        cmd.env("TEXINPUTS", &texinputs);
        // Pipe the output
        cmd.stderr(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped());
        // What to do with the output
        clam::Options::apply(self.cli_options, &mut cmd);
        // The actual input to the tex program
        cmd.arg(dirs::START_FILE);
        Engine { cmd }
    }
}

#[derive(Debug, Clone, Serialize)]
#[allow(unused)]
pub enum InteractionMode {
    BatchMode,
    NonStopMode,
    ScrollMode,
    ErrorStopMode,
}

impl clam::ArgValue for InteractionMode {
    fn set_cmd_arg<C: clam::Command>(&self, name: &str, cmd: &mut C) {
        let mode = match self {
            InteractionMode::BatchMode => "batchmode",
            InteractionMode::NonStopMode => "nonstopmode",
            InteractionMode::ScrollMode => "scrollmode",
            InteractionMode::ErrorStopMode => "errorstopmode",
        };
        cmd.args([name, mode]);
    }
}

#[derive(Debug, Clone, Serialize)]
#[allow(unused)]
pub enum MkTexFormat {
    Tex,
    Tfm,
    Pk,
}

impl clam::ArgValue for MkTexFormat {
    fn set_cmd_arg<C: clam::Command>(&self, name: &str, cmd: &mut C) {
        let format = match self {
            MkTexFormat::Tex => "tex",
            MkTexFormat::Tfm => "tfm",
            MkTexFormat::Pk => "pk",
        };
        cmd.args([name, format]);
    }
}

#[derive(Debug, Clone, Serialize)]
#[allow(unused)]
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
    fn set_cmd_arg<C: clam::Command>(&self, name: &str, cmd: &mut C) {
        let special = match self {
            SrcSpecial::Cr => "cr",
            SrcSpecial::Display => "display",
            SrcSpecial::Hbox => "hbox",
            SrcSpecial::Math => "math",
            SrcSpecial::Par => "par",
            SrcSpecial::Parend => "parend",
            SrcSpecial::Vbox => "vbox",
        };
        cmd.args([name, special]);
    }
}

#[derive(Debug, Clone, Serialize)]
#[allow(unused)]
pub enum Format {
    Pdf,
    Dvi,
}

impl clam::ArgValue for Format {
    fn set_cmd_arg<C: clam::Command>(&self, name: &str, cmd: &mut C) {
        let format = match self {
            Format::Pdf => "pdf",
            Format::Dvi => "dvi",
        };
        cmd.args([name, format]);
    }
}

pub type ConfigurationFileLine = String;

pub type TcxName = String;

/// Syntex option type
pub type SynctexNumber = i32;

pub const SYNCTEX_GZIPPED: SynctexNumber = 1;

#[allow(unused)]
pub const SYNCTEX_UNZIPPED: SynctexNumber = -1;

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
    interaction: Option<InteractionMode>,
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
    shell_escape: bool,
    /// disable \write18{SHELL COMMAND}
    no_shell_escape: bool,
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
