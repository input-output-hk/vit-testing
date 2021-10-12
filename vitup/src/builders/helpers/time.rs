use crate::config::VitStartParameters;
use crate::config::VoteBlockchainTime;
use crate::config::VoteTime;
use chrono::NaiveDateTime;
use chrono::Utc;
use jormungandr_lib::time::SecondsSinceUnixEpoch;
use std::collections::HashSet;

pub fn convert_to_blockchain_date(
    parameters: &VitStartParameters,
    block0_date: SecondsSinceUnixEpoch,
) -> VoteBlockchainTime {
    match parameters.vote_time {
        VoteTime::Blockchain(blockchain_date) => blockchain_date,
        VoteTime::Real {
            vote_start_timestamp,
            tally_start_timestamp,
            tally_end_timestamp,
            find_best_match: _,
        } => {
            let block0_date = NaiveDateTime::from_timestamp(block0_date.to_secs() as i64, 0);
            let from_block0_till_vote_start = (vote_start_timestamp - block0_date).num_seconds();
            let from_vote_start_till_tally_start =
                (tally_start_timestamp - block0_date).num_seconds();
            let from_tally_start_till_tally_end = (tally_end_timestamp - block0_date).num_seconds();

            let mut block0_to_start = count_divisors(from_block0_till_vote_start, 90, 60);
            let start_to_end = count_divisors(from_vote_start_till_tally_start, 90, 60);

            block0_to_start.extend(&start_to_end);
            let max = block0_to_start.iter().max();
            if max.is_none() {
                panic!("bad data");
            }

            let max: i64 = *max.unwrap();

            VoteBlockchainTime {
                vote_start: (from_block0_till_vote_start / max) as u32,
                tally_start: (from_vote_start_till_tally_start / max) as u32,
                tally_end: (from_tally_start_till_tally_end / max) as u32,
                slots_per_epoch: max as u32,
            }
        }
    }
}

fn count_divisors(n: i64, grace: i64, start: i64) -> HashSet<i64> {
    let mut output = HashSet::new();
    for i in start..=n {
        if n % i <= grace {
            output.insert(i);
        }
    }
    output
}

pub fn convert_to_human_date(
    parameters: &VitStartParameters,
    block0_date: SecondsSinceUnixEpoch,
) -> (NaiveDateTime, NaiveDateTime, NaiveDateTime) {
    let parameters = parameters.clone();

    println!(
        "Current date {:?}",
        NaiveDateTime::from_timestamp(block0_date.to_secs() as i64, 0)
    );

    match parameters.vote_time {
        VoteTime::Blockchain(blockchain) => {
            let epoch_duration = parameters.slot_duration as u32 * blockchain.slots_per_epoch;
            let vote_start_timestamp =
                block0_date.to_secs() as u32 + epoch_duration * blockchain.vote_start;
            let vote_start_timestamp =
                NaiveDateTime::from_timestamp(vote_start_timestamp as i64, 0);
            let tally_start_timestamp =
                block0_date.to_secs() as u32 + epoch_duration * blockchain.tally_start;
            let tally_start_timestamp =
                NaiveDateTime::from_timestamp(tally_start_timestamp as i64, 0);
            let tally_end_timestamp =
                block0_date.to_secs() as u32 + epoch_duration * blockchain.tally_end;
            let tally_end_timestamp = NaiveDateTime::from_timestamp(tally_end_timestamp as i64, 0);

            (
                vote_start_timestamp,
                tally_start_timestamp,
                tally_end_timestamp,
            )
        }
        VoteTime::Real {
            vote_start_timestamp,
            tally_start_timestamp,
            tally_end_timestamp,
            find_best_match: _,
        } => (
            vote_start_timestamp,
            tally_start_timestamp,
            tally_end_timestamp,
        ),
    }
}

pub fn default_snapshot_date() -> NaiveDateTime {
    let dt = Utc::now();
    NaiveDateTime::from_timestamp((dt - chrono::Duration::hours(3)).timestamp(), 0)
}

pub fn default_next_vote_date() -> NaiveDateTime {
    let dt = Utc::now();
    NaiveDateTime::from_timestamp((dt + chrono::Duration::days(30)).timestamp(), 0)
}

pub fn default_next_snapshot_date() -> NaiveDateTime {
    let dt = Utc::now();
    NaiveDateTime::from_timestamp((dt + chrono::Duration::days(29)).timestamp(), 0)
}

/*
TODO uncomment once implementation is provided

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_test() {
        let block0_date = SecondsSinceUnixEpoch::now();
        let mut parameters = VitStartParameters::default();

        let vote_time = VoteTime::real_from_str(
            "2021-10-06 11:00:00",
            "2021-10-06 18:00:00",
            "2021-10-07 09:00:00",
        )
        .unwrap();

        parameters.slot_duration = 10;
        parameters.vote_time = vote_time.clone();
        println!("Before {:#?}", vote_time);
        let blockchain_time = convert_to_blockchain_date(&parameters, block0_date);
        parameters.vote_time = VoteTime::Blockchain(blockchain_time);
        println!(
            "After {:#?}",
            convert_to_human_date(&parameters, block0_date)
        );
    }
}
*/
