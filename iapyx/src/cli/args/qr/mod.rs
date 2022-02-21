mod address;
mod verify;

use address::GetAddressFromQrCommand;
use jormungandr_lib::interfaces::Block0ConfigurationError;
use structopt::StructOpt;
use thiserror::Error;
use verify::VerifyQrCommand;

#[derive(StructOpt, Debug)]
pub enum IapyxQrCommand {
    Verify(VerifyQrCommand),
    CheckAddress(GetAddressFromQrCommand),
}

impl IapyxQrCommand {
    pub fn exec(&self) -> Result<(), IapyxQrCommandError> {
        match self {
            Self::Verify(verify) => verify.exec(),
            Self::CheckAddress(check_address) => check_address.exec(),
        }
    }
}

#[derive(Error, Debug)]
pub enum IapyxQrCommandError {
    #[error("pin error")]
    PinError(#[from] crate::qr::PinReadError),
    #[error("reqwest error")]
    IapyxQrCommandError(#[from] reqwest::Error),
    #[error("block0 parse error")]
    Block0ParseError(#[from] Block0ConfigurationError),
    #[error("io error")]
    IoError(#[from] std::io::Error),
    #[error("read error")]
    ReadError(#[from] chain_core::property::ReadError),
    #[error("bech32 error")]
    Bech32Error(#[from] bech32::Error),
}
