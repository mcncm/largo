use serde::Serialize;

use super::{private::CommandBuilder, Engine, EngineBuilder};
use crate::dirs;

pub struct PdflatexBuilder {
    cmd: crate::build::Command,
    /// The `\input{main.tex}` that should terminate the tex input
    input: String,
    cli_options: CommandLineOptions,
}

impl CommandBuilder for PdflatexBuilder {
    fn inner_cmd(&self) -> &crate::build::Command {
        &self.cmd
    }

    fn inner_cmd_mut(&mut self) -> &mut crate::build::Command {
        &mut self.cmd
    }
}

impl PdflatexBuilder {
    // NOTE: Only using `conf` just to find its own executable. In fact, it
    // should probably be using some _other_ input; that's more data than it
    // should have access to.
    pub fn new(conf: &crate::conf::LargoConfig) -> Self {
        let cmd = crate::build::Command::new(&conf.executables.pdflatex);
        let mut cli_options = CommandLineOptions::default();
        // Always use nonstop mode for now.
        cli_options.interaction = Some(InteractionMode::NonStopMode);
        Self {
            cmd,
            cli_options,
            input: String::new(),
        }
    }

    fn disable_line_wrapping(&mut self) {
        // FIXME: you should be able to do this as a static converstion to a
        // &'static str, and without an allocation.
        self.cmd.env("max_print_line", &i32::MAX.to_string());
    }

    fn finish_input(&mut self) {
        self.input.push_str(r#"\input{"#);
        self.input.push_str(dirs::MAIN_FILE);
        self.input.push('}');
    }
}

impl EngineBuilder for PdflatexBuilder {
    fn with_output_dir<P: typedir::AsPath<dirs::ProfileBuildDir>>(mut self, path: P) -> Self {
        self.cli_options.output_directory = Some(path.to_path_buf());
        self
    }

    fn with_verbosity(self, _verbosity: &crate::build::Verbosity) -> Self {
        // FIXME: just a no-op for now
        self
    }

    fn with_synctex(mut self, use_synctex: bool) -> anyhow::Result<Self> {
        if use_synctex {
            self.cli_options.synctex = Some(SYNCTEX_GZIPPED);
        }
        Ok(self)
    }

    // FIXME: Just do this with macros.
    fn with_largo_vars(mut self, vars: &crate::vars::LargoVars) -> anyhow::Result<Self> {
        use std::fmt::Write;
        write!(self.input, r#"\def\LargoProfile{{{}}}"#, vars.profile)?;
        write!(
            self.input,
            r#"\def\LargoOutputDirectory{{{}}}"#,
            vars.output_directory.display()
        )?;
        if let Some(bib) = &vars.bibliography {
            write!(self.input, r#"\def\LargoBibliography{{{}}}"#, bib)?;
        }
        Ok(self)
    }

    fn with_shell_escape(mut self, shell_escape: Option<bool>) -> anyhow::Result<Self> {
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
        self.disable_line_wrapping();
        self.finish_input();

        let mut cmd = self.cmd;
        cmd.stderr(smol::process::Stdio::piped())
            .stdout(smol::process::Stdio::piped());
        // What to do with the output
        clam::Options::apply(self.cli_options, &mut cmd);
        // The actual input to the tex program
        cmd.arg(&self.input);
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
        cmd.args([name, &mode]);
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
        cmd.args([name, &format]);
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
        cmd.args([name, &special]);
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
        cmd.args([name, &format]);
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
