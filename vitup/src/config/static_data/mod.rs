mod current;
mod next;

pub use current::CurrentFund;
pub use next::NextFund;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct StaticData {
    pub current_fund: CurrentFund,
    pub next_funds: Vec<NextFund>,
}
