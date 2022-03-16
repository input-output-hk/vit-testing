use crate::Vote;

use crate::common::iapyx_from_secret_key;
use assert_fs::TempDir;
use catalyst_toolbox::rewards::voters::calculate_rewards;
use catalyst_toolbox::rewards::voters::vote_count_with_addresses;
use chain_impl_mockchain::block::BlockDate;
use jormungandr_automation::testing::time;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vitup::builders::utils::DeploymentTree;
use vitup::config::VoteBlockchainTime;
use vitup::config::{ConfigBuilder, InitialEntry, Initials};
use vitup::testing::spawn_network;
use vitup::testing::vitup_setup;

const PIN: &str = "1234";
const ALICE: &str = "alice";
const BOB: &str = "bob";
const CLARICE: &str = "clarice";

#[test]
pub fn voters_with_at_least_one_vote() {
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let stake = 10_000;
    let vote_timing = VoteBlockchainTime {
        vote_start: 0,
        tally_start: 1,
        tally_end: 2,
        slots_per_epoch: 30,
    };
    let config = ConfigBuilder::default()
        .initials(Initials(vec![
            InitialEntry::Wallet {
                name: ALICE.to_string(),
                funds: stake,
                pin: PIN.to_string(),
                role: Default::default(),
            },
            InitialEntry::Wallet {
                name: BOB.to_string(),
                funds: stake,
                pin: PIN.to_string(),
                role: Default::default(),
            },
            InitialEntry::Wallet {
                name: CLARICE.to_string(),
                funds: stake,
                pin: PIN.to_string(),
                role: Default::default(),
            },
        ]))
        .vote_timing(vote_timing.into())
        .slot_duration_in_seconds(2)
        .proposals_count(3)
        .voting_power(100)
        .private(false)
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

    let deployment_tree = DeploymentTree::new(testing_directory.path());

    let mut alice =
        iapyx_from_secret_key(deployment_tree.wallet_secret(ALICE), &wallet_proxy).unwrap();
    let mut bob = iapyx_from_secret_key(deployment_tree.wallet_secret(BOB), &wallet_proxy).unwrap();

    let fund1_vote_plan = &controller.defined_vote_plans()[0];

    alice
        .vote_for(fund1_vote_plan.id(), 0, Vote::Yes as u8)
        .unwrap();

    bob.vote_for(fund1_vote_plan.id(), 1, Vote::Yes as u8)
        .unwrap();

    bob.vote_for(fund1_vote_plan.id(), 0, Vote::Yes as u8)
        .unwrap();

    let target_date = BlockDate {
        epoch: 1,
        slot_id: 0,
    };
    time::wait_for_date(target_date.into(), nodes[0].rest());

    let block0 = &controller.settings().block0;
    let records = calculate_rewards(
        vote_count_with_addresses(nodes[0].rest().account_votes_count().unwrap(), block0),
        block0,
        1,
        100,
    )
    .unwrap();

    let alice_reward_record = records
        .iter()
        .find(|x| x.address = alice.address())
        .unwrap();

    assert_eq!(alice_reward_record.stake, stake);
    assert_eq!(alice_reward_record.voter_reward_lovelace, "33");

    let bob_reward_record = records.iter().find(|x| x.address = bob.address()).unwrap();

    assert_eq!(alice_reward_record.stake, stake);
    assert_eq!(alice_reward_record.voter_reward_lovelace, "33");

    let clarice_reward_record = records
        .iter()
        .find(|x| x.address = clarice.address())
        .unwrap();

    assert_eq!(clarice_reward_record.stake, stake);
    assert_eq!(clarice_reward_record.voter_reward_lovelace, "0");
}
