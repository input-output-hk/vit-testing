use catalyst_toolbox::snapshot::voting_group::VotingGroupAssigner;
use catalyst_toolbox::snapshot::RawSnapshot;
use catalyst_toolbox::snapshot::Snapshot;
use catalyst_toolbox::snapshot::VotingRegistration;
use fraction::Fraction;
use jortestkit::prelude::Wait;
use snapshot_trigger_service::client::rest::SnapshotRestClient;
use snapshot_trigger_service::config::Configuration;
use snapshot_trigger_service::config::JobParameters;
use std::process::Child;

pub struct SnapshotServiceController {
    child: Child,
    configuration: Configuration,
    client: SnapshotRestClient,
}

impl SnapshotServiceController {
    pub fn new(child: Child, configuration: Configuration) -> Self {
        Self {
            child,
            client: SnapshotRestClient::new(format!("http://127.0.0.1:{}", configuration.port)),
            configuration,
        }
    }

    pub fn client(&self) -> &SnapshotRestClient {
        &self.client
    }

    pub fn shutdown(mut self) -> Result<(), std::io::Error> {
        self.child.kill()
    }

    pub fn configuration(&self) -> &Configuration {
        &self.configuration
    }

    pub fn snapshot(
        &self,
        job_params: JobParameters,
        threshold: u64,
        fraction: Fraction,
        voting_group_assigner: &impl VotingGroupAssigner,
    ) -> Snapshot {
        let id = self.client().job_new(job_params.clone()).unwrap();

        self.client()
            .wait_for_job_finish(&id, Wait::new(std::time::Duration::from_secs(10), 5))
            .unwrap();

        let snapshot_content = self
            .client()
            .get_snapshot(id, job_params.tag.as_ref().unwrap().to_string())
            .unwrap();
        let raw_snapshot: Vec<VotingRegistration> =
            serde_json::from_str(&snapshot_content).unwrap();

        Snapshot::from_raw_snapshot(
            RawSnapshot::from(raw_snapshot),
            threshold.into(),
            fraction,
            voting_group_assigner,
        )
        .unwrap()
    }
}

impl Drop for SnapshotServiceController {
    fn drop(&mut self) {
        // There's no kill like overkill
        let _ = self.child.kill();
        // FIXME: These should be better done in a test harness
        self.child.wait().unwrap();
    }
}