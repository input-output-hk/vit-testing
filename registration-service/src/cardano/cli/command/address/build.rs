use crate::config::NetworkType;
use crate::utils::CommandExt;
use std::path::Path;
use std::process::Command;
pub struct AddressBuildCommand {
    command: Command,
}

impl AddressBuildCommand {
    pub fn new(command: Command) -> Self {
        Self { command }
    }

    pub fn verification_key_file<P: AsRef<Path>>(mut self, verification_key: P) -> Self {
        self.command
            .arg("--verification-key-file")
            .arg(verification_key.as_ref());
        self
    }

    pub fn stake_verification_key_file<P: AsRef<Path>>(
        mut self,
        stake_verification_key: P,
    ) -> Self {
        self.command
            .arg("--stake-verification-key-file")
            .arg(stake_verification_key.as_ref());
        self
    }

    pub fn out_file<P: AsRef<Path>>(mut self, out_file: P) -> Self {
        self.command.arg("--out-file").arg(out_file.as_ref());
        self
    }

    pub fn network(mut self, network: NetworkType) -> Self {
        self.command.arg_network(network);
        self
    }

    pub fn build(self) -> Command {
        println!("Cardano Cli - address build: {:?}", self.command);
        self.command
    }
}
