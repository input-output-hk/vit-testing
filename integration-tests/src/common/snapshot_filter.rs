use catalyst_toolbox::snapshot::{RawSnapshot, Snapshot, VotingRegistration};
use chain_addr::Discrimination;
use jormungandr_lib::interfaces::InitialUTxO;
use jormungandr_lib::interfaces::Value;
use snapshot_trigger_service::client::SnapshotResult;
use voting_hir::VoterHIR;

pub struct SnapshotFilter {
    snapshot: Snapshot,
}

impl SnapshotFilter {
    pub fn from_snapshot_result(
        snapshot_result: &SnapshotResult,
        voting_threshold: Value,
    ) -> SnapshotFilter {
        Self {
            snapshot: Snapshot::from_raw_snapshot(
                RawSnapshot::from(snapshot_result.registrations().to_vec()),
                voting_threshold,
            ),
        }
    }

    pub fn to_voters_hirs(&self) -> Vec<VoterHIR> {
        self.snapshot
            .voting_keys()
            .map(|vk| VoterHIR {
                voting_key: vk.clone(),
                voting_power: self
                    .snapshot
                    .contributions_for_voting_key(vk)
                    .iter()
                    .map(|c| c.value)
                    .sum::<u64>()
                    .into(),
                voting_group: "direct".to_string(),
            })
            .collect()
    }

    pub fn to_block0_initials(&self) -> Vec<InitialUTxO> {
        self.snapshot.to_block0_initials(Discrimination::Production)
    }
}
