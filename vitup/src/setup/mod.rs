pub mod convert;
pub mod diff;
pub mod generate;
pub mod start;
pub mod validate;

use crate::error::Result;
use crate::mock::MockStartCommandArgs;
use crate::setup::generate::CommitteeIdCommandArgs;
use crate::setup::generate::{QrCommandArgs, SnapshotCommandArgs};
use crate::setup::start::AdvancedStartCommandArgs;
use convert::ConvertCommand;
use diff::DiffCommand;
use generate::DataCommandArgs;
use start::QuickStartCommandArgs;
use structopt::StructOpt;
use validate::ValidateCommand;

#[derive(StructOpt, Debug)]
pub enum VitCliCommand {
    /// start backend
    Start(StartCommand),
    /// generate fund data
    Generate(GenerateCommand),
    // get diff between new deployment and target env
    Diff(DiffCommand),
    // validate data
    Validate(ValidateCommand),
    // convert data
    Convert(ConvertCommand),
}

impl VitCliCommand {
    pub fn exec(self) -> Result<()> {
        match self {
            Self::Start(start_command) => start_command.exec(),
            Self::Generate(generate_command) => generate_command.exec(),
            Self::Diff(diff_command) => diff_command.exec(),
            Self::Validate(validate_command) => validate_command.exec(),
            Self::Convert(convert_command) => convert_command.exec(),
        }
    }
}

#[derive(StructOpt, Debug)]
pub enum StartCommand {
    /// start backend from scratch
    Quick(QuickStartCommandArgs),
    /// start advanced backend from scratch
    Advanced(AdvancedStartCommandArgs),
    // start mock env
    Mock(MockStartCommandArgs),
}

impl StartCommand {
    pub fn exec(self) -> Result<()> {
        match self {
            Self::Quick(quick_start_command) => quick_start_command.exec(),
            Self::Advanced(advanced_start_command) => advanced_start_command.exec(),
            Self::Mock(mock_start_command) => mock_start_command.exec().map_err(Into::into),
        }
    }
}

#[derive(StructOpt, Debug)]
pub enum GenerateCommand {
    /// generate qrs
    Qr(QrCommandArgs),
    /// generate data only
    Data(DataCommandArgs),
    /// generate snapshot data only
    Snapshot(SnapshotCommandArgs),
    /// Committee Id
    Committee(CommitteeIdCommandArgs),
}

impl GenerateCommand {
    pub fn exec(self) -> Result<()> {
        match self {
            Self::Qr(quick_start_command) => quick_start_command.exec(),
            Self::Data(data_start_command) => data_start_command.exec(),
            Self::Snapshot(snapshot_start_command) => snapshot_start_command.exec(),
            Self::Committee(generate_committee_command) => generate_committee_command.exec(),
        }
    }
}
