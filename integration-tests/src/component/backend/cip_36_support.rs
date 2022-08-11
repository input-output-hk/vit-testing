use crate::common::snapshot::do_snapshot;
use crate::common::SnapshotFilter;
use crate::Vote;
use assert_fs::TempDir;
use chain_addr::Discrimination;
use chain_impl_mockchain::block::BlockDate;
use chain_impl_mockchain::key::Hash;
use hersir::builder::VotePlanSettings;
use jormungandr_automation::testing::asserts::VotePlanStatusAssert;
use jormungandr_automation::testing::time;
use snapshot_trigger_service::config::JobParameters;
use std::path::Path;
use std::str::FromStr;
use thor::FragmentSender;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vitup::config::ConfigBuilder;
use vitup::config::SnapshotInitials;
use vitup::config::VoteBlockchainTime;
use vitup::config::{Block0Initial, Block0Initials};
use vitup::testing::spawn_network;
use vitup::testing::vitup_setup;
use voting_hir::VoterHIR;

#[test]
pub fn cip_36_support() {
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let voting_threshold = 1;
    let tag = None;

    let job_param = JobParameters {
        slot_no: None,
        tag: tag.clone(),
    };

    let snapshot_result = do_snapshot(job_param).unwrap();
    let registrations = snapshot_result.registrations();

    let snapshot_filter =
        SnapshotFilter::from_snapshot_result(&snapshot_result, voting_threshold.into());

    let config = ConfigBuilder::default()
        .voting_power(voting_threshold)
        .block0_initials(Block0Initials::new_from_external_utxo(
            snapshot_filter.to_block0_initials(),
        ))
        .snapshot_initials(SnapshotInitials::from_voters_hir(
            snapshot_filter.to_voters_hirs(),
            tag.unwrap_or("".to_string()),
        ))
        .build();

    let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();

    let (mut controller, vit_parameters, network_params) =
        vitup_setup(&config, testing_directory.path().to_path_buf()).unwrap();
    let (nodes, _vit_station, wallet_proxy) = spawn_network(
        &mut controller,
        vit_parameters,
        network_params,
        &mut template_generator,
    )
    .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(3600))
}
