use crate::common::registration::RegistrationServiceStarter;
use crate::common::MainnetWallet;
use assert_fs::TempDir;
use catalyst_toolbox::kedqr;
use chain_crypto::{Ed25519Extended, SecretKey};
use mainnet_tools::cardano_cli::CardanoCliMock;
use mainnet_tools::voter_registration::VoterRegistrationMock;
use registration_service::client::RegistrationResult;
use registration_service::config::{ConfigurationBuilder, NetworkType};

#[test]
pub fn registration_flow() {
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let stake = 10_000;

    let alice = MainnetWallet::new(stake);

    let voter_registration_mock = VoterRegistrationMock::default();
    let cardano_cli_mock = CardanoCliMock::default();

    let configuration = ConfigurationBuilder::default()
        .with_cardano_cli(cardano_cli_mock.path())
        .with_voter_registration(voter_registration_mock.path())
        .with_network(NetworkType::Mainnet)
        .with_tmp_result_dir(&testing_directory)
        .build();

    let registration_service = RegistrationServiceStarter::default()
        .with_configuration(configuration)
        .start_on_available_port(&testing_directory)
        .unwrap();

    let direct_voting_registration = alice.direct_voting_registration();
    voter_registration_mock.with_response(direct_voting_registration, &testing_directory);

    let registration_result = registration_service.register(&alice, &testing_directory);
    let key_qr_code = get_secret_key_from_qr_code(registration_result);

    assert_eq!(
        alice.catalyst_secret_key().leak_secret().as_ref(),
        key_qr_code.leak_secret().as_ref()
    );
}

fn get_secret_key_from_qr_code(
    registration_result: RegistrationResult,
) -> SecretKey<Ed25519Extended> {
    let img = image::open(registration_result.qr_code()).unwrap();
    //TODO: send pin to registration service or extract it from qr code filename
    let secrets = kedqr::KeyQrCode::decode(img, &[1, 2, 3, 4]).unwrap();
    let key_qr_code = secrets.get(0).unwrap().clone();
    key_qr_code
}
