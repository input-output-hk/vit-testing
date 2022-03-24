use crate::client::rest::VitupAdminRestClient;
use crate::client::rest::VitupDisruptionRestClient;
use crate::client::rest::VitupRest;
use crate::config::Config;
use crate::Result;
use std::path::PathBuf;
use structopt::StructOpt;
use thor::PersistentLogViewer;

#[derive(StructOpt, Debug)]
pub struct VitupClientCommand {
    #[structopt(short, long, env = "VIT_TOKEN")]
    token: Option<String>,

    #[structopt(short, long, env = "VIT_ENDPOINT")]
    endpoint: String,

    #[structopt(subcommand)]
    command: Command,
}

impl VitupClientCommand {
    pub fn exec(self) -> Result<()> {
        if let Command::Utils(command) = self.command {
            return command.exec();
        }
        let endpoint = self.endpoint;
        let rest = match self.token {
            Some(token) => VitupRest::new_with_token(token, endpoint),
            None => VitupRest::new(endpoint),
        };

        match self.command {
            Command::Disruption(disruption_command) => disruption_command.exec(rest.into()),
            Command::Mock(mock_command) => mock_command.exec(rest.into()),
            _ => panic!("should not happen"),
        }
    }
}

#[derive(StructOpt, Debug)]
pub enum Command {
    /// disruption
    Disruption(DisruptionCommand),
    /// mock
    Mock(MockCommand),
    /// utils
    Utils(UtilsCommand),
}

#[derive(StructOpt, Debug)]
pub enum DisruptionCommand {
    /// start backend from scratch
    Logs(LogsCommand),
    /// start advanced backend from scratch
    Files(FilesCommand),
    // start mock env
    Control(ControlCommand),
}

impl DisruptionCommand {
    pub fn exec(self, rest: VitupDisruptionRestClient) -> Result<()> {
        match self {
            Self::Logs(logs_command) => logs_command.exec(rest),
            Self::Files(files_command) => files_command.exec(rest),
            Self::Control(control_command) => control_command.exec(rest),
        }
    }
}

#[derive(StructOpt, Debug)]
pub enum LogsCommand {
    /// start backend from scratch
    Clear,
    /// start advanced backend from scratch
    Get,
}

impl LogsCommand {
    pub fn exec(self, rest: VitupDisruptionRestClient) -> Result<()> {
        match self {
            Self::Clear => rest.clear_logs().map_err(Into::into),
            Self::Get => {
                println!("{:?}", rest.get_logs());
                Ok(())
            }
        }
    }
}

#[derive(StructOpt, Debug)]
pub enum FilesCommand {
    List,
}

impl FilesCommand {
    pub fn exec(self, rest: VitupDisruptionRestClient) -> Result<()> {
        match self {
            Self::List => {
                println!("{}", serde_json::to_string_pretty(&rest.list_files()?)?);
                Ok(())
            }
        }
    }
}

#[derive(StructOpt, Debug)]
pub enum ControlCommand {
    Reset,
    SetUnavailable,
    SetErrorCode(SetErrorCodeCommand),
    SetAvailable,
    SetFundId(SetFundIdCommand),
    Fragments(FragmentsCommand),
    Health,
}

impl ControlCommand {
    pub fn exec(self, rest: VitupDisruptionRestClient) -> Result<()> {
        match self {
            Self::Reset => rest.reset().map_err(Into::into),
            Self::SetUnavailable => rest.make_unavailable().map_err(Into::into),
            Self::SetErrorCode(set_error_code) => {
                rest.set_error_code(set_error_code.code).map_err(Into::into)
            }
            Self::SetAvailable => rest.make_available().map_err(Into::into),
            Self::SetFundId(set_fund_id) => {
                rest.set_fund_id(set_fund_id.fund_id).map_err(Into::into)
            }
            Self::Fragments(fragments_command) => fragments_command.exec(rest).map_err(Into::into),
            Self::Health => {
                match rest.is_up() {
                    true => {
                        println!("env is up");
                    }
                    false => {
                        println!("env is down");
                    }
                };
                Ok(())
            }
        }
    }
}

#[derive(StructOpt, Debug)]
pub struct SetFundIdCommand {
    #[structopt(long = "fund-id")]
    fund_id: u32,
}

#[derive(StructOpt, Debug)]
pub struct SetErrorCodeCommand {
    #[structopt(long = "code")]
    code: u16,
}

#[derive(StructOpt, Debug)]
pub enum FragmentsCommand {
    Reject,
    Hold,
    Accept,
    Reset,
}

impl FragmentsCommand {
    pub fn exec(self, rest: VitupDisruptionRestClient) -> Result<()> {
        match self {
            Self::Reject => rest.reject_all_fragments().map_err(Into::into),
            Self::Hold => rest.hold_all_fragments().map_err(Into::into),
            Self::Accept => rest.accept_all_fragments().map_err(Into::into),
            Self::Reset => rest.reset_fragments_behavior().map_err(Into::into),
        }
    }
}

#[derive(StructOpt, Debug)]
pub enum MockCommand {
    /// files commands
    Files(MockFilesCommand),
    /// start commands
    Start(MockStartCommand),
    /// stop command
    Stop,
    /// status command
    Status,
}

#[derive(StructOpt, Debug)]
pub enum MockStartCommand {
    /// start custom
    Custom(MockStartCustomCommand),
    /// start default
    Standard,
}

impl MockStartCommand {
    pub fn exec(self, rest: VitupAdminRestClient) -> Result<()> {
        match self {
            Self::Custom(custom_start) => {
                custom_start.exec(rest)?;
                Ok(())
            }
            Self::Standard => {
                println!("{}", rest.start_default()?);
                Ok(())
            }
        }
    }
}

#[derive(StructOpt, Debug)]
pub struct MockStartCustomCommand {
    #[structopt(short = "p", long = "params")]
    params: std::path::PathBuf,
}

impl MockStartCustomCommand {
    pub fn exec(self, rest: VitupAdminRestClient) -> Result<()> {
        let content = jortestkit::prelude::read_file(self.params);
        let params: Config = serde_json::from_str(&content)?;
        println!("{}", rest.start_custom(params)?);
        Ok(())
    }
}

#[derive(StructOpt, Debug)]
pub enum MockFilesCommand {
    List,
}

impl MockFilesCommand {
    pub fn exec(self, rest: VitupAdminRestClient) -> Result<()> {
        match self {
            Self::List => {
                println!("{}", serde_json::to_string_pretty(&rest.list_files()?)?);
                Ok(())
            }
        }
    }
}

impl MockCommand {
    pub fn exec(self, rest: VitupAdminRestClient) -> Result<()> {
        match self {
            Self::Files(files_command) => files_command.exec(rest),
            Self::Start(start_command) => {
                start_command.exec(rest)?;
                Ok(())
            }
            Self::Stop => {
                println!("{}", rest.stop()?);
                Ok(())
            }
            Self::Status => {
                println!("{}", rest.status()?);
                Ok(())
            }
        }
    }
}

#[derive(StructOpt, Debug)]
pub enum UtilsCommand {
    /// persistent log comamnds
    PersistentLog(PersistentLogCommand),
}

impl UtilsCommand {
    pub fn exec(self) -> Result<()> {
        match self {
            Self::PersistentLog(persistent_logs_command) => persistent_logs_command.exec(),
        }
    }
}

#[derive(StructOpt, Debug)]
pub enum PersistentLogCommand {
    /// persistent log commands
    Count(CountPersistentLogCommand),
}

impl PersistentLogCommand {
    pub fn exec(self) -> Result<()> {
        match self {
            Self::Count(count_command) => count_command.exec(),
        }
    }
}

#[derive(StructOpt, Debug)]
pub struct CountPersistentLogCommand {
    /// count commands  
    #[structopt(long = "folder")]
    pub folder: PathBuf,
}

impl CountPersistentLogCommand {
    pub fn exec(self) -> Result<()> {
        println!("{}", PersistentLogViewer::new(self.folder).count());
        Ok(())
    }
}
