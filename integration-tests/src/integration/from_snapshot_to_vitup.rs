use catalyst_toolbox::snapshot::voting_group::RepsVotersAssigner;
use catalyst_toolbox::snapshot::Delegations;
use crate::common::snapshot::SnapshotServiceStarter;
use crate::common::MainnetWallet;
use assert_fs::TempDir;
use fraction::Fraction;
use mainnet_tools::db_sync::DbSyncInstance;
use mainnet_tools::network::MainnetNetwork;
use std::collections::HashSet;
use mainnet_tools::voting_tools::VotingToolsMock;
use snapshot_trigger_service::config::ConfigurationBuilder;
use jormungandr_automation::testing::block0::read_initials;
use snapshot_trigger_service::config::JobParameters;
use voting_hir::VoterHIR;
use vitup::config::ConfigBuilder;

const DIRECT_VOTING_GROUP: &str = "direct";
const REP_VOTING_GROUP: &str = "rep";

#[test]
pub fn cip36_mixed_delegation_should_appear_in_block0() {
    let testing_directory = TempDir::new().unwrap().into_persistent();


    let stake = 10_000;

    let alice_voter = MainnetWallet::new(stake);
    let bob_voter = MainnetWallet::new(stake);
    let clarice_voter = MainnetWallet::new(stake);

    let david_representative = MainnetWallet::new(500);
    let edgar_representative = MainnetWallet::new(1_000);
    let fred_representative = MainnetWallet::new(8_000);

    let mut reps = HashSet::new();
    reps.insert(edgar_representative.catalyst_public_key());
    reps.insert(david_representative.catalyst_public_key());
    reps.insert(fred_representative.catalyst_public_key());

    let mut mainnet_network = MainnetNetwork::default();
    let mut db_sync_instance = DbSyncInstance::default();

    mainnet_network.sync_with(&mut db_sync_instance);

    alice_voter
        .send_direct_voting_registration()
        .to(&mut mainnet_network);
        bob_voter
        .send_voting_registration(Delegations::New(
            vec![(david_representative.catalyst_public_key(),1)]
        ))
        .to(&mut mainnet_network);
    clarice_voter
        .send_voting_registration(Delegations::New(
            vec![
                (david_representative.catalyst_public_key(),1),
                (edgar_representative.catalyst_public_key(),1)
            ]
        ))
        .to(&mut mainnet_network);

    let voting_tools =
        VotingToolsMock::default().connect_to_db_sync(&db_sync_instance, &testing_directory);

    let configuration = ConfigurationBuilder::default()
        .with_voting_tools_params(voting_tools.into())
        .with_tmp_result_dir(&testing_directory)
        .build();


    let assigner = RepsVotersAssigner::new_from_repsdb(DIRECT_VOTING_GROUP.to_string(), REP_VOTING_GROUP.to_string(), reps).unwrap();

    let voter_hir = SnapshotServiceStarter::default()
        .with_configuration(configuration)
        .start(&testing_directory)
        .unwrap()
        .snapshot(JobParameters::fund("fund9"), 450u64, Fraction::from(1u64),&assigner)
        .to_voter_hir();
    

    let config = ConfigBuilder::default().build();
    config.initials.block0.extend_from_external(read_initials(voter_hir).unwrap());
    println!("{:?}",config);
}

pub fn write_config<P: AsRef<Path>>(config: Configuration, path: P) -> Result<(), Error> {
    use std::io::Write;
    let mut file = std::fs::File::create(&path)?;
    file.write_all(serde_json::to_string(&config)?.as_bytes())
        .map_err(Into::into)
}