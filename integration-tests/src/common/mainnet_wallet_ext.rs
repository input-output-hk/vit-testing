use crate::common::MainnetWallet;
use vitup::config::Block0Initial;

pub trait MainnetWalletExtension {
    fn as_initial_entry(&self) -> Block0Initial;
}

impl MainnetWalletExtension for MainnetWallet {
    fn as_initial_entry(&self) -> Block0Initial {
        Block0Initial::External {
            address: self.catalyst_address().to_string(),
            funds: self.stake(),
            role: Default::default(),
        }
    }
}
