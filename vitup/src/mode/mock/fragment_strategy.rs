use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum FragmentRecieveStrategy {
    Reject,
    Accept,
    Pending,
    None,
    //For cases when we want to implement mempool cleaning
    Forget,
}

pub struct FragmentRecieveStrategyChain {
    default_strategy: FragmentRecieveStrategy,
    chain: VecDeque<FragmentRecieveStrategy>,
}

impl Default for FragmentRecieveStrategyChain {
    fn default() -> Self {
        Self::new(FragmentRecieveStrategy::None)
    }
}

impl FragmentRecieveStrategyChain {
    pub fn new(default_strategy: FragmentRecieveStrategy) -> Self {
        Self {
            default_strategy,
            chain: VecDeque::new(),
        }
    }

    pub fn pop(&mut self) -> FragmentRecieveStrategy {
        self.chain.pop_front().unwrap_or(self.default_strategy)
    }

    pub fn set_default(&mut self, default_strategy: FragmentRecieveStrategy) {
        self.default_strategy = default_strategy
    }

    pub fn set_chain(&mut self, chain: VecDeque<FragmentRecieveStrategy>) {
        self.chain = chain
    }
}
