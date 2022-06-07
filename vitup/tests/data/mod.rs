mod private;
mod public;

use std::collections::LinkedList;
use valgrind::{Challenge, Fund, ValgrindClient};
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;
use vit_servicing_station_tests::common::data::ChallengeTemplate;
use vit_servicing_station_tests::common::data::FundTemplate;
use vit_servicing_station_tests::common::data::ProposalTemplate;
use vit_servicing_station_tests::common::data::ReviewTemplate;

pub fn funds_eq(expected: FundTemplate, actual: Fund) {
    assert_eq!(expected.id, actual.id, "fund id");
    assert_eq!(expected.goal, actual.fund_goal, "fund goal");
    assert_eq!(
        expected.threshold.unwrap() as i64 * 1_000_000,
        actual.voting_power_threshold,
        "threshold"
    );
}

pub fn challenges_eq(expected_list: LinkedList<ChallengeTemplate>, actual_list: Vec<Challenge>) {
    if expected_list.len() != actual_list.len() {
        panic!("challenges count invalid");
    }

    for (expected, actual) in expected_list.iter().zip(actual_list.iter()) {
        assert_eq!(expected.id, actual.id.to_string(), "id");
        assert_eq!(
            expected.challenge_type.to_string(),
            actual.challenge_type.to_string(),
            "challenge type"
        );
        assert_eq!(expected.title, actual.title, "title");
        assert_eq!(expected.description, actual.description, "description");
        assert_eq!(
            expected.rewards_total,
            actual.rewards_total.to_string(),
            "rewards total"
        );
        assert_eq!(
            expected.proposers_rewards,
            actual.proposers_rewards.to_string(),
            "proposer rewards"
        );
        assert_eq!(
            expected.challenge_url, actual.challenge_url,
            "challenge url"
        );
        assert_eq!(
            expected.fund_id.as_ref().unwrap().to_string(),
            actual.fund_id.to_string(),
            "fund id"
        );
    }
}

pub fn reviews_eq(expected_list: LinkedList<ReviewTemplate>, backend_client: ValgrindClient) {
    for expected in expected_list.iter() {
        for actuals in backend_client
            .review(&expected.proposal_id)
            .unwrap()
            .values()
        {
            for actual in actuals {
                assert_eq!(
                    expected.proposal_id.to_string(),
                    actual.proposal_id.to_string(),
                    "proposal id"
                );
                assert_eq!(
                    expected.impact_alignment_rating_given, actual.impact_alignment_rating_given,
                    "impact alignment rating given"
                );
                assert_eq!(
                    expected.impact_alignment_note, actual.impact_alignment_note,
                    "impact alignment note"
                );
                assert_eq!(
                    expected.feasibility_rating_given, actual.feasibility_rating_given,
                    "feasibility rating given"
                );
                assert_eq!(
                    expected.feasibility_note, actual.feasibility_note,
                    "feasibility note"
                );
                assert_eq!(
                    expected.impact_alignment_rating_given, actual.impact_alignment_rating_given,
                    "impact alignment rating given"
                );
                assert_eq!(
                    expected.auditability_rating_given, actual.auditability_rating_given,
                    "auditability rating given"
                );
                assert_eq!(
                    expected.auditability_note, actual.auditability_note,
                    "auditability note"
                );
                assert_eq!(expected.assessor, actual.assessor, "assessor");
            }
        }
    }
}

pub fn proposals_eq(
    expected_list: LinkedList<ProposalTemplate>,
    actual_list: Vec<FullProposalInfo>,
) {
    if expected_list.len() != actual_list.len() {
        panic!("proposals count invalid");
    }

    for (expected, actual) in expected_list.iter().zip(actual_list.iter()) {
        assert_eq!(
            expected.internal_id,
            actual.proposal.internal_id.to_string(),
            "internal id"
        );
        assert_eq!(
            expected.category_name, actual.proposal.proposal_category.category_name,
            "category name"
        );
        assert_eq!(
            expected.proposal_title, actual.proposal.proposal_title,
            "proposal title"
        );
        assert_eq!(
            expected.proposal_summary, actual.proposal.proposal_summary,
            "proposal summary"
        );
        assert_eq!(
            expected.proposal_funds,
            actual.proposal.proposal_funds.to_string(),
            "proposal funds"
        );
        assert_eq!(
            expected.proposal_url, actual.proposal.proposal_url,
            "proposal url"
        );
        assert_eq!(
            expected.proposal_impact_score,
            actual.proposal.proposal_impact_score.to_string(),
            "proposal impact score"
        );
        assert_eq!(
            expected.proposer_name, actual.proposal.proposer.proposer_name,
            "proposer name"
        );
        assert_eq!(
            expected.proposer_url, actual.proposal.proposer.proposer_url,
            "proposer url"
        );
        assert_eq!(
            expected.proposer_relevant_experience,
            actual.proposal.proposer.proposer_relevant_experience,
            "proposer relevant experience"
        );
        assert_eq!(
            expected.chain_vote_options.as_csv_string(),
            actual.proposal.chain_vote_options.as_csv_string(),
            "chain vote options"
        );
        assert_eq!(
            expected.chain_vote_type, actual.proposal.chain_voteplan_payload,
            "chain vote type"
        );
        assert_eq!(
            expected.challenge_id.as_ref().unwrap(),
            &actual.proposal.challenge_id.to_string(),
            "challenge id"
        );
    }
}
