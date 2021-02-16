pub enum VitupCliCommand {

    ///
    /// Set the directory path where vitupcli will install. Mainly remember to set `$VITUPCLI_HOME/bin` value to
    /// your $PATH for easy access to the default release's tools.
    #[structopt(long)]
    jorup_home: Option<PathBuf>,

    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Generate autocompletion scripts for the given <SHELL>
    ///
    /// Generate the autocompletion scripts for the given shell, Autocompletion
    /// will be written in the standard output and can then be pasted by the
    /// user to the appropriate place.
    Completions {
        shell: structopt::clap::Shell,
    },

   // Control(control::Command),
   // Files(files::Command),
    Info(info::Command),
    Status(status::Command),
    Setup(setup::Command),
}

impl VitupCliCommand {
    pub fn exec(self) -> Result<()> {
        match self {
            Self::Start(start_command) => start_command.exec(),
            Self::Generate(generate_command) => generate_command.exec(),
        }
    }
}
