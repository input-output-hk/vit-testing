use assert_fs::TempDir;
use chain_addr::Discrimination;
use chain_impl_mockchain::tokens::identifier::TokenIdentifier as ChainTokenId;
use jormungandr_lib::interfaces::Block0Configuration;
use jormungandr_lib::interfaces::Destination;
use jormungandr_lib::interfaces::Initial;
use jormungandr_lib::interfaces::TokenIdentifier;
use std::path::PathBuf;
use std::str::FromStr;
use valgrind::ValgrindClient;
use vit_servicing_station_tests::common::data::parse_funds;
use vit_servicing_station_tests::common::data::ExternalValidVotingTemplateGenerator;
use vitup::builders::utils::DeploymentTree;
use vitup::config::Block0Initial;
use vitup::config::Block0Initials;
use vitup::config::Role;
use vitup::config::{ConfigBuilder, VoteBlockchainTime};
use vitup::testing::{spawn_network, vitup_setup};

#[test]
pub fn representative_multiple_vote_plans() {
    let funds_path = PathBuf::from_str("./resources/example/funds.json").unwrap();
    let mut template_generator = ExternalValidVotingTemplateGenerator::new(
        PathBuf::from_str("./resources/example/proposals.json").unwrap(),
        PathBuf::from_str("./resources/example/challenges.json").unwrap(),
        funds_path.clone(),
        PathBuf::from_str("./resources/example/review.json").unwrap(),
    )
    .unwrap();
    let expected_funds = parse_funds(funds_path).unwrap();
    let funds = 1000;
    let mut rnd = rand::rngs::OsRng;
    let alice = thor::Wallet::new_account(&mut rnd, Discrimination::Production);
    let bob = thor::Wallet::new_account(&mut rnd, Discrimination::Production);

    if expected_funds.len() > 1 {
        panic!("more than 1 expected fund is not supported");
    }
    let expected_fund = expected_funds.iter().next().unwrap().clone();

    let testing_directory = TempDir::new().unwrap().into_persistent();

    let vote_timing = VoteBlockchainTime {
        vote_start: 0,
        tally_start: 1,
        tally_end: 2,
        slots_per_epoch: 30,
    };

    let config = ConfigBuilder::default()
        .vote_timing(vote_timing.into())
        .fund_id(expected_fund.id)
        .slot_duration_in_seconds(2)
        .proposals_count(template_generator.proposals_count() as u32)
        .challenges_count(template_generator.challenges_count() as usize)
        .reviews_count(3)
        .voting_power(expected_fund.threshold.unwrap() as u64)
        .private(true)
        .block0_initials(Block0Initials(vec![
            Block0Initial::External {
                address: alice.address().to_string(),
                funds,
                role: Role::Voter,
            },
            Block0Initial::External {
                address: alice.address().to_string(),
                funds,
                role: Role::Representative,
            },
        ]))
        .build();

    let (mut controller, vit_parameters, network_params) =
        vitup_setup(&config, testing_directory.path().to_path_buf()).unwrap();
    let (_nodes, _vit_station, wallet_proxy) = spawn_network(
        &mut controller,
        vit_parameters,
        network_params,
        &mut template_generator,
    )
    .unwrap();

    let files_tree = DeploymentTree::new(testing_directory.path());

    let contents = std::fs::read_to_string(&files_tree.voting_token()).unwrap();
    let voting_tokens: Vec<(Role, TokenIdentifier)> = serde_json::from_str(&contents).unwrap();

    println!("{:?}", voting_tokens.iter().cloned().map(|(r, t)| (r, t)));

    let contents = std::fs::read_to_string(&files_tree.genesis_path()).unwrap();
    let block_configuration: Block0Configuration = serde_json::from_str(&contents).unwrap();

    let backend_client = ValgrindClient::new(wallet_proxy.address(), Default::default()).unwrap();

    println!("{:?}", backend_client.vit_client().proposals("direct"));
    println!("{:?}", backend_client.vit_client().proposals("dreps"));

    for initial in block_configuration.initial {
        if let Initial::Token(token) = initial {
            let chain_token: ChainTokenId = token.token_id.into();

            if token.to.contains(&Destination {
                address: alice.address(),
                value: funds.into(),
            }) {
                println!(
                    "alice: {} -> {} ",
                    alice.address(),
                    hex::encode(&chain_token.token_name)
                );
            }
            if token.to.contains(&Destination {
                address: bob.address(),
                value: funds.into(),
            }) {
                println!(
                    "bob: {} -> {} ",
                    bob.address(),
                    hex::encode(&chain_token.token_name)
                );
            }
        }
    }
}
