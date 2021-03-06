use crate::{
    config::{read_config, Configuration},
    context::{Context, ContextLock},
    service::ManagerService,
};

use crate::job::RegistrationVerifyJobBuilder;
use futures::future::FutureExt;
use std::sync::Mutex;
use std::{path::PathBuf, sync::Arc};
use structopt::StructOpt;
use thiserror::Error;

#[derive(StructOpt, Debug)]
pub struct RegistrationVerifyServiceCommand {
    #[structopt(long = "api-token")]
    pub api_token: Option<String>,

    #[structopt(long = "admin-token")]
    pub admin_token: Option<String>,

    #[structopt(long = "config")]
    pub config: PathBuf,
}

impl RegistrationVerifyServiceCommand {
    pub async fn exec(self) -> Result<(), Error> {
        let mut configuration: Configuration = read_config(&self.config)?;

        if self.api_token.is_some() {
            configuration.client_token = self.api_token;
        }

        if self.admin_token.is_some() {
            configuration.admin_token = self.admin_token;
        }

        let control_context: ContextLock =
            Arc::new(Mutex::new(Context::new(configuration.clone())));

        let mut manager = ManagerService::new(control_context.clone());
        let handle = manager.spawn();

        let request_to_start_task = async {
            loop {
                if let Some((_job_id, request)) = manager.request_to_start() {
                    let job = RegistrationVerifyJobBuilder::new()
                        .with_jcli(&configuration.jcli)
                        .with_snapshot_token(&configuration.snapshot_token)
                        .with_snapshot_address(&configuration.snapshot_address)
                        .with_network(configuration.network)
                        .build();

                    control_context.lock().unwrap().run_started().unwrap();
                    let output_info = job.start(request, control_context.clone()).unwrap();

                    control_context
                        .lock()
                        .unwrap()
                        .run_finished(output_info)
                        .unwrap();
                }
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
        .fuse();

        tokio::pin!(request_to_start_task);

        futures::select! {
            result = request_to_start_task => result,
            _ = handle.fuse() => Ok(()),
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot spawn configuration")]
    CannotSpawnCommand(#[from] std::io::Error),
    #[error("cannot read configuration")]
    CannotReadConfiguration(#[from] crate::config::Error),
    #[error("context error")]
    Context(#[from] crate::context::Error),
}
