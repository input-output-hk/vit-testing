use crate::config::NetworkType;
use crate::utils::CommandExt;
use std::path::Path;
use std::process::Command;

pub struct QueryCommand {
    command: Command,
}

impl QueryCommand {
    pub fn new(command: Command) -> Self {
        Self { command }
    }

    pub fn tip(mut self, network: NetworkType) -> Self {
        self.command.arg("tip").arg_network(network);
        self
    }

    pub fn utxo<S: Into<String>>(mut self, network: NetworkType, payment_address: S) -> Self {
        self.command
            .arg("utxo")
            .arg_network(network)
            .arg("--address")
            .arg(payment_address.into())
            .arg("--out-file")
            .arg("/dev/stdout");
        self
    }

    pub fn build(self) -> Command {
        self.command
    }
}
