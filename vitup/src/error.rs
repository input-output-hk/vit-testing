use crate::scenario::vit_station::Error as VitStationControllerError;
use crate::scenario::wallet::WalletProxyError;
use crate::wallet::WalletProxyControllerError;
use hersir::controller::NodeError;
use jormungandr_automation::testing::ConsumptionBenchmarkError;
use jormungandr_automation::testing::VerificationError;
use jormungandr_lib::interfaces::Block0ConfigurationError;
use jormungandr_lib::interfaces::FragmentStatus;
use std::time::Duration;
use thor::FragmentSenderError;
use thor::FragmentVerifierError;
use thor::WalletError;
use vit_servicing_station_tests::common::startup::server::ServerBootstrapperError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Interactive(#[from] jortestkit::console::InteractiveCommandError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    Node(#[from] NodeError),
    #[error(transparent)]
    Wallet(#[from] WalletError),
    #[error(transparent)]
    FragmentSender(#[from] FragmentSenderError),
    #[error(transparent)]
    FragmentVerifier(#[from] FragmentVerifierError),
    #[error(transparent)]
    VerificationFailed(#[from] VerificationError),
    #[error(transparent)]
    MonitorResourcesError(#[from] ConsumptionBenchmarkError),
    #[error(transparent)]
    VitStationControllerError(#[from] VitStationControllerError),
    #[error(transparent)]
    WalletProxyError(#[from] WalletProxyError),
    #[error(transparent)]
    TemplateLoadError(#[from] vit_servicing_station_tests::common::data::TemplateLoad),
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    #[error(transparent)]
    SerdeYamlError(#[from] serde_yaml::Error),
    #[error(transparent)]
    Block0EncodeError(#[from] chain_impl_mockchain::ledger::Error),
    #[error(transparent)]
    ImageReadError(#[from] image::error::ImageError),
    #[error(transparent)]
    MockError(#[from] crate::cli::start::MockError),
    #[error(transparent)]
    ParseError(#[from] chrono::ParseError),
    #[error(transparent)]
    ClientRestError(#[from] crate::client::rest::Error),
    #[error(transparent)]
    Block0ConfigurationError(#[from] Block0ConfigurationError),
    #[error(transparent)]
    VitServerBootstrapperError(#[from] ServerBootstrapperError),
    #[error(transparent)]
    VitRestError(#[from] vit_servicing_station_tests::common::clients::RestError),
    #[error(transparent)]
    ChainAddressError(#[from] chain_addr::Error),
    #[error(transparent)]
    ChainBech32Error(#[from] chain_crypto::bech32::Error),
    #[error(transparent)]
    GlobError(#[from] glob::GlobError),
    #[error(transparent)]
    ValgrindError(#[from] valgrind::Error),
    #[error(transparent)]
    ImportError(#[from] crate::cli::import::ImportError),
    #[error(transparent)]
    Validate(#[from] crate::cli::ValidateError),
    #[error(transparent)]
    ControllerError(#[from] hersir::controller::Error),
    #[error(transparent)]
    WalletProxyController(#[from] WalletProxyControllerError),
    #[error("synchronization for nodes has failed. {}. Timeout was: {} s", info, timeout.as_secs())]
    SyncTimeoutOccurred { info: String, timeout: Duration },
    #[error("{info}")]
    AssertionFailed { info: String },
    #[error(
        "transaction should be 'In Block'. status: {:?}, node: {}",
        status,
        node
    )]
    TransactionNotInBlock {
        node: String,
        status: FragmentStatus,
    },
    #[error("proxy with alias: {alias} not found")]
    ProxyNotFound { alias: String },
    #[error("unknown log level: {0}")]
    UnknownLogLevel(String),
    #[error("environment is down")]
    EnvironmentIsDown,
    #[error("wrong format for snapshot data")]
    SnapshotIntialReadError,
    #[error("no challenge id found for proposal {proposal_id}")]
    NoChallengeIdFound { proposal_id: String },
}
