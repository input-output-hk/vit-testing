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
            println!(
                "from_block0_till_vote_start: {:?}",
                from_block0_till_vote_start
            );
            println!(
                "from_vote_start_till_tally_start: {:?}",
                from_vote_start_till_tally_start
            );
            println!(
                "from_tally_start_till_tally_end: {:?}",
                from_tally_start_till_tally_end
            );

            let block0_to_start = count_divisors(from_block0_till_vote_start, 90, 1800);
            let start_to_tally = count_divisors(from_vote_start_till_tally_start, 90, 1800);
            let tally_to_end = count_divisors(from_tally_start_till_tally_end, 90, 1800);

            println!("block0_to_start: {:?}", block0_to_start);
            println!("start_to_tally: {:?}", start_to_tally);
            println!("tally_to_end: {:?}", tally_to_end);

            let intersection = find_common_divisors(&block0_to_start, &start_to_tally, 90);
            let intersection = find_common_divisors(&intersection, &tally_to_end, 90);

            let max: i64 = *intersection.iter().max().expect("no max");
            let slot_duration: i64 = parameters.slot_duration.into();
            println!("max: {:?}", max);
            VoteBlockchainTime {
                vote_start: (from_block0_till_vote_start / (max)) as u32,
                tally_start: (from_vote_start_till_tally_start / (max)) as u32,
                tally_end: (from_tally_start_till_tally_end / (max)) as u32,
                slots_per_epoch: (max / slot_duration) as u32,
            }
        }
    }
}

fn find_common_divisors(left: &HashSet<i64>, right: &HashSet<i64>, grace: i64) -> HashSet<i64> {
    let mut output = HashSet::new();
    for left_item in left {
        if right.iter().any(|x| left_item - *x < grace) {
            output.insert(*left_item);
        }
    }
    output
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_test() {
        //2022-01-11 8:00:00
        //let block0_date = SecondsSinceUnixEpoch::from_secs(1641888000);
        let block0_date = SecondsSinceUnixEpoch::now();
        let mut parameters = VitStartParameters::default();

        let vote_time = VoteTime::real_from_str(
            "2022-01-11 11:30:00",
            "2022-01-11 18:00:00",
            "2022-01-12 09:00:00",
        )
        .unwrap();

        parameters.slot_duration = 2;
        parameters.vote_time = vote_time.clone();
        println!("Before {:#?}", vote_time);
        let blockchain_time = convert_to_blockchain_date(&parameters, block0_date);
        parameters.vote_time = VoteTime::Blockchain(blockchain_time);
        println!("Blockchain: {:#?}", parameters.vote_time);
        println!(
            "After {:#?}",
            convert_to_human_date(&parameters, block0_date)
        );
    }
}
