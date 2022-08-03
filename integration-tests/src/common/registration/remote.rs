use registration_service::{
    client::{do_registration, rest::RegistrationRestClient},
    config::Configuration,
};

use assert_fs::TempDir;
use iapyx::utils::qr::SecretFromQrCode;
use mainnet_tools::wallet::MainnetWallet;
use registration_service::client::RegistrationResult;
use registration_service::request::Request;

pub struct RemoteRegistrationServiceController {
    configuration: Configuration,
    client: RegistrationRestClient,
}

impl RemoteRegistrationServiceController {
    pub fn new(configuration: Configuration) -> Self {
        Self {
            client: RegistrationRestClient::new(format!("http://127.0.0.1:{}", configuration.port)),
            configuration,
        }
    }

    pub fn client(&self) -> &RegistrationRestClient {
        &self.client
    }

    pub fn configuration(&self) -> &Configuration {
        &self.configuration
    }

    pub fn register(&self, wallet: &MainnetWallet, temp_dir: &TempDir) -> RegistrationResult {
        let key = wallet.leak_key();
        let registration_request = Request {
            payment_skey: key.payment_skey_cbor_hex(),
            payment_vkey: key.payment_vkey_cbor_hex(),
            stake_skey: key.stake_skey_cbor_hex(),
            stake_vkey: key.stake_vkey_cbor_hex(),
            vote_skey: Some(wallet.catalyst_secret_key().to_bech32().unwrap()),
        };

        println!("{:?}", registration_request);
        do_registration(registration_request, self.client(), temp_dir)
    }
}
