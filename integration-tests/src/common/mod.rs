pub mod asserts;
mod backend;
pub mod load;
pub mod registration;
pub mod snapshot;
pub use backend::*;

use jormungandr_testing_utils::testing::node::time;
use jormungandr_testing_utils::testing::node::JormungandrRest;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("vitup error")]
    VitupError(#[from] vitup::error::Error),
    #[error("node error")]
    NodeError(#[from] jormungandr_scenario_tests::node::Error),
    #[error("verification error")]
    VerificationError(#[from] jormungandr_testing_utils::testing::VerificationError),
    #[error("sender error")]
    FragmentSenderError(#[from] jormungandr_testing_utils::testing::FragmentSenderError),
    #[error("scenario error")]
    ScenarioError(#[from] jormungandr_scenario_tests::scenario::Error),
    #[error("iapyx error")]
    IapyxError(#[from] iapyx::ControllerError),
}

#[allow(dead_code)]
pub enum Vote {
    Blank = 0,
    Yes = 1,
    No = 2,
}

#[allow(dead_code)]
pub struct VoteTiming {
    pub vote_start: u32,
    pub tally_start: u32,
    pub tally_end: u32,
}

impl VoteTiming {
    pub fn new(vote_start: u32, tally_start: u32, tally_end: u32) -> Self {
        Self {
            vote_start,
            tally_start,
            tally_end,
        }
    }

    pub fn wait_for_tally_start(self, rest: JormungandrRest) {
        time::wait_for_epoch(self.tally_start, rest);
    }

    pub fn wait_for_tally_end(self, rest: JormungandrRest) {
        time::wait_for_epoch(self.tally_end, rest);
    }
}
