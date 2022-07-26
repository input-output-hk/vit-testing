mod builder;
mod info;

use crate::cardano::CardanoCli;
use crate::cardano::CardanoKeyTemplate;
use crate::config::NetworkType;
use crate::request::Request;
use crate::utils::write_content;
use crate::utils::CommandExt as _;
use crate::Error;
use crate::VoterRegistrationCli;
pub use builder::VoteRegistrationJobBuilder;
pub use info::JobOutputInfo;
use jormungandr_automation::jcli::JCli;
use jortestkit::prelude::read_file;
use jortestkit::prelude::ProcessOutput;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const PIN: &str = "1234";

pub struct VoteRegistrationJob {
    pub(super) jcli: PathBuf,
    pub(super) cardano_cli: CardanoCli,
    pub(super) voter_registration: VoterRegistrationCli,
    pub(super) network: NetworkType,
    pub(super) working_dir: PathBuf,
}

impl VoteRegistrationJob {
    pub fn start(&self, request: Request) -> Result<JobOutputInfo, Error> {
        println!("saving payment.skey...");
        let payment_skey = CardanoKeyTemplate::payment_signing_key(request.payment_skey);
        let payment_skey_path = Path::new(&self.working_dir).join("payment.skey");
        payment_skey.write_to_file(&payment_skey_path)?;
        println!("payment.skey saved");

        println!("saving payment.vkey...");
        let payment_vkey = CardanoKeyTemplate::payment_verification_key(request.payment_vkey);
        let payment_vkey_path = Path::new(&self.working_dir).join("payment.vkey");
        payment_vkey.write_to_file(&payment_vkey_path)?;
        println!("payment.vkey saved");

        println!("saving stake.skey...");
        let stake_skey = CardanoKeyTemplate::stake_signing_key(request.stake_skey);
        let stake_skey_path = Path::new(&self.working_dir).join("stake.skey");
        stake_skey.write_to_file(&stake_skey_path)?;
        println!("stake.skey saved");

        println!("saving stake.vkey...");
        let stake_vkey = CardanoKeyTemplate::stake_verification_key(request.stake_vkey);
        let stake_vkey_path = Path::new(&self.working_dir).join("stake.vkey");
        stake_vkey.write_to_file(&stake_vkey_path)?;
        println!("stake.vkey saved");

        let jcli = JCli::new(self.jcli.clone());
        let private_key = if let Some(key) = request.vote_skey {
            key
        } else {
            jcli.key().generate_default()
        };

        println!("saving catalyst-vote.skey...");
        let private_key_path = Path::new(&self.working_dir).join("catalyst-vote.skey");
        write_content(&private_key, &private_key_path)?;
        println!("catalyst-vote.skey saved");

        println!("saving catalyst-vote.pkey...");
        let public_key = jcli.key().convert_to_public_string(&private_key);
        let public_key_path = Path::new(&self.working_dir).join("catalyst-vote.pkey");
        write_content(&public_key, &public_key_path)?;
        println!("catalyst-vote.pkey saved");

        println!("saving rewards.addr...");
        let rewards_address_path = Path::new(&self.working_dir).join("rewards.addr");
        self.cardano_cli.stake_address().build(
            &stake_vkey_path,
            &rewards_address_path,
            self.network,
        )?;
        println!("rewards.addr saved");

        let rewards_address = read_file(&rewards_address_path)?;
        println!("rewards.addr: {}", rewards_address);

        println!("saving payment.addr...");
        let payment_address_path = Path::new(&self.working_dir).join("payment.addr");
        self.cardano_cli.address().build(
            &payment_vkey_path,
            &stake_vkey_path,
            &payment_address_path,
            self.network,
        )?;
        println!("payment.addr saved");

        let payment_address = read_file(&payment_address_path)?;
        println!("payment.addr: {}", payment_address);

        let funds = self
            .cardano_cli
            .query()
            .funds(&payment_address, self.network)?;

        let slot: u64 = self.cardano_cli.query().tip(self.network)?.parse().unwrap();

        let metadata_path = Path::new(&self.working_dir).join("metadata.json");

        let voter_registration = self.voter_registration.clone();

        voter_registration.generate_metadata(
            rewards_address,
            public_key_path,
            stake_skey_path,
            slot,
            metadata_path,
        )?;

        //   self.cardano_cli.transaction().sign(sign_transaction);
        //   self.cardano_cli.transaction().submit();

        Ok(JobOutputInfo {
            slot_no: slot,
            funds,
        })
    }

    /*

        let mut tx_signed_file = Path::new(&self.working_dir).join("tx.signed");
        let id = Uuid::new_v4();
        let mut tx_signed_file = self.config.result_dir.clone();
        tx_signed_file.push(id.to_string());
        tx_signed_file.push("tx.signed");

        std::fs::create_dir_all(
            tx_signed_file
                .parent()
                .ok_or_else(|| Error::CannotGetParentDirectory(tx_signed_file.to_path_buf()))?,
        )
        .map_err(|x| Error::CannotCreateParentDirectory(x.to_string()))?;

        let mut file =
            File::create(&tx_signed_file).map_err(|x| Error::CannotCreateAFile(x.to_string()))?;
        file.write_all(&raw)
            .map_err(|x| Error::CannotWriteAFile(x.to_string()))?;
    */
}
