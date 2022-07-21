use crate::common::get_available_port;
use crate::common::snapshot::SnapshotServiceController;
use assert_fs::fixture::PathChild;
use assert_fs::TempDir;
use snapshot_trigger_service::config::write_config;
use snapshot_trigger_service::config::Configuration;
use std::path::Path;
use std::path::PathBuf;

use super::Error;
use std::process::Command;

pub struct SnapshotServiceStarter {
    configuration: Configuration,
    path_to_bin: PathBuf,
}

impl Default for SnapshotServiceStarter {
    fn default() -> Self {
        Self {
            configuration: Default::default(),
            path_to_bin: Path::new("snapshot-trigger-service").to_path_buf(),
        }
    }
}

impl SnapshotServiceStarter {
    pub fn with_configuration(mut self, configuration: Configuration) -> Self {
        self.configuration = configuration;
        self
    }

    pub fn with_path_to_bin<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.path_to_bin = path.as_ref().to_path_buf();
        self
    }

    pub fn start_on_available_port(
        mut self,
        temp_dir: &TempDir,
    ) -> Result<SnapshotServiceController, Error> {
        self.configuration.port = get_available_port();
        self.start(temp_dir)
    }

    pub fn start(self, temp_dir: &TempDir) -> Result<SnapshotServiceController, Error> {
        let config_file = temp_dir.child("snapshot_trigger_service_config.yaml");
        write_config(self.configuration.clone(), config_file.path())?;
        Ok(SnapshotServiceController::new(
            Command::new(self.path_to_bin)
                .arg("--config")
                .arg(config_file.path())
                .spawn()?,
            self.configuration,
        ))
    }
}
