pub mod args;
pub mod rest;

use crate::{client::rest::RegistrationRestClient, context::State, request::Request};
use assert_fs::fixture::PathChild;
use assert_fs::TempDir;
use jormungandr_automation::jcli::JCli;
use jormungandr_lib::crypto::account::Identifier;
use jormungandr_lib::interfaces::Value;
use jortestkit::prelude::WaitBuilder;
use math::round;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

pub fn do_registration(
    request: Request,
    registration_client: &RegistrationRestClient,
    temp_dir: &TempDir,
) -> RegistrationResult {
    let registration_job_id = registration_client.job_new(request).unwrap();

    let wait = WaitBuilder::new().tries(10).sleep_between_tries(10).build();
    println!("waiting for registration job");
    let registration_jobs_status = registration_client
        .wait_for_job_finish(registration_job_id.clone(), wait)
        .unwrap();
    println!("{:?}", registration_jobs_status);

    let qr_code_path = temp_dir.child("qr_code");
    std::fs::create_dir_all(qr_code_path.path()).unwrap();

    let qr_code = registration_client
        .download_qr(registration_job_id.clone(), qr_code_path.path())
        .unwrap();
    let voting_key_sk = registration_client
        .get_catalyst_sk(registration_job_id)
        .unwrap();

    RegistrationResult {
        status: registration_jobs_status,
        qr_code,
        voting_sk: voting_key_sk,
    }
}

#[derive(Debug)]
pub struct RegistrationResult {
    status: State,
    qr_code: PathBuf,
    voting_sk: String,
}

impl RegistrationResult {
    pub fn assert_status_is_finished(&self) {
        matches!(self.status, State::Finished { .. });
    }

    pub fn status(&self) -> State {
        self.status.clone()
    }

    pub fn qr_code(&self) -> PathBuf {
        self.qr_code.clone()
    }

    pub fn pin(&self) -> String {
        let chars: Vec<char> = self
            .qr_code
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .chars()
            .rev()
            .take(4)
            .collect();
        chars.iter().rev().collect()
    }

    pub fn snapshot_entry(&self) -> Result<(Identifier, Value), Error> {
        Ok((self.identifier()?, self.funds_in_ada()?.into()))
    }

    pub fn print_snapshot_entry(&self) -> Result<(), Error> {
        println!(
            "[identifier: {}, funds:{}",
            self.identifier_as_str(),
            self.funds_in_ada()?
        );
        Ok(())
    }

    pub fn identifier_as_str(&self) -> String {
        let jcli = JCli::new(PathBuf::from_str("jcli").expect("jcli not found on env"));
        jcli.key().convert_to_public_string(&self.voting_sk)
    }

    pub fn identifier(&self) -> Result<Identifier, Error> {
        Ok(Identifier::from_str(&self.identifier_as_str())?)
    }

    pub fn slot_no(&self) -> Result<u64, Error> {
        match self.status() {
            State::Finished { info, .. } => Ok(info.slot_no),
            _ => Err(Error::CannotGetSlotNoFromRegistrationResult),
        }
    }

    pub fn funds_in_ada(&self) -> Result<u64, Error> {
        match self.status() {
            State::Finished { info, .. } => {
                let rounded = round::floor(info.funds as f64, -6);
                Ok((rounded as u64) / 1_000_000)
            }
            _ => Err(Error::CannotGetFundsFromRegistrationResult),
        }
    }

    pub fn funds_in_lovelace(&self) -> Result<u64, Error> {
        match self.status() {
            State::Finished { info, .. } => Ok(info.funds),
            _ => Err(Error::CannotGetFundsFromRegistrationResult),
        }
    }

    pub fn leak_sk(&self) -> String {
        self.voting_sk.clone()
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot get funds from registration result")]
    CannotGetFundsFromRegistrationResult,
    #[error("cannot get address from registration result")]
    CannotGetAddressFromRegistrationResult(#[from] chain_addr::Error),
    #[error("cannot get slot no from registration result")]
    CannotGetSlotNoFromRegistrationResult,
    #[error(transparent)]
    Bech32(#[from] chain_crypto::bech32::Error),
}
