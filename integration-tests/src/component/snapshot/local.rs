use crate::common::registration::do_registration;
use crate::common::snapshot::do_snapshot;
use crate::common::snapshot::wait_for_db_sync;
use crate::common::snapshot::VoterHIRAsserts;
use assert_fs::TempDir;
use snapshot_trigger_service::config::JobParameters;
const GRACE_PERIOD_FOR_SNAPSHOT: u64 = 300;

#[test]
pub fn mixed_registration_transactions() {
    let temp_dir = TempDir::new().unwrap().into_persistent();

    let first_registartion = do_registration(&temp_dir);
    first_registartion.assert_status_is_finished();
    first_registartion.assert_qr_equals_to_sk();

    let (overriden_id, _) = first_registartion.snapshot_entry().unwrap();

    println!("Waiting 10 mins before running next registration");
    std::thread::sleep(std::time::Duration::from_secs(5 * 60));
    println!("Wait finished.");

    let second_registartion = do_registration(&temp_dir);
    second_registartion.assert_status_is_finished();
    second_registartion.assert_qr_equals_to_sk();

    let (correct_id, value) = second_registartion.snapshot_entry().unwrap();

    let job_param = JobParameters {
        slot_no: Some(second_registartion.slot_no().unwrap() + GRACE_PERIOD_FOR_SNAPSHOT),
        tag: None,
    };

    wait_for_db_sync();
    let snapshot_result = do_snapshot(job_param).unwrap();
    let initials = snapshot_result.initials();

    initials.assert_contains_voting_key_and_value(&correct_id, value);
    initials.assert_not_contain_voting_key(&overriden_id);
}
