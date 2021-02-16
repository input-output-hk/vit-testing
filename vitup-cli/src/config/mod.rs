mod environment;

#[derive(Debug)]
pub struct VitupCliConfig {
    home_dir: PathBuf,
    environments: Option<environment::Config>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("No $HOME environment variable, can not set VITUP_CLI_HOME value.")]
    NoHomeDir,
    #[error("Cannot create VITUP_CLI_HOME: {1}")]
    CannotCreateHomeDir(#[source] io::Error, PathBuf),
    #[error("Cannot create directory: {1}")]
    CannotCreateInitDir(#[source] io::Error, PathBuf),
    #[error("Cannot open file: {1}")]
    CannotOpenFile(#[source] io::Error, PathBuf),
    #[error("Cannot parse file: {1}")]
    Json(#[source] serde_json::Error, PathBuf),
}

impl VitupCliConfig {
    pub fn new(
        jorup_home: Option<PathBuf>,
        offline: bool,
    ) -> Result<Self, Error> {
        let home_dir = jorup_home
            .or_else(|| dirs::home_dir().map(|d| d.join(".vitup-cli")))
            .ok_or_else(|| Error::NoHomeDir)?;

        let home_dir = if home_dir.is_absolute() {
            home_dir
        } else {
            std::env::current_dir().unwrap().join(home_dir)
        };

        std::fs::create_dir_all(&home_dir)
            .map_err(|e| Error::CannotCreateHomeDir(e, home_dir.clone()))?;

        let cfg = VitupCliConfig {
            home_dir,
            environments: None,
        };
        Ok(cfg)
    }

    pub fn vitup_cli_file(&self) -> PathBuf {
        self.home_dir.join("vitup-cli.json")
    }

    pub fn load_config(&mut self) -> Result<&definitions::Config, Error> {
        if self.environments.is_none() {
            let file = std::fs::File::open(self.vitup_cli_file()).map_err(|e| {
                eprintln!("HINT: run `vitup-cli setup install`");
                Error::CannotOpenFile(e, self.jorfile())
            })?;

            let environments = serde_json::from_reader(file).map_err(|e| Error::Json(e, self.vitup_cli_file()))?;
            self.environments = Some(environments);
        }
        Ok(self.environments.as_ref().unwrap())
    }
}