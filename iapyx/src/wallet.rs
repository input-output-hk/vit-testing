use bip39::{dictionary, Entropy, Type};
use chain_addr::{AddressReadable, Discrimination};
use chain_core::{
    packer::Codec,
    property::{Deserialize, Fragment as _},
};
use chain_impl_mockchain::{
    block::BlockDate,
    fragment::{Fragment, FragmentId},
    transaction::Input,
};
use hdkeygen::account::AccountId;
use jormungandr_lib::interfaces::AccountIdentifier;
use std::str::FromStr;
use thiserror::Error;
use wallet::Settings;
use wallet_core::Proposal;
use wallet_core::Wallet as Inner;
use wallet_core::{Choice, Value};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot recover from mnemonics: {0}")]
    CannotRecover(String),
    #[error("cannot retrieve funds: {0}")]
    CannotRetrieveFunds(String),
    #[error("backend error")]
    BackendError(#[from] valgrind::Error),
    #[error("cannot send vote")]
    CannotSendVote(String),
}

pub struct Wallet {
    inner: Inner,
}

impl Wallet {
    pub fn generate(words_length: Type) -> Result<Self, Error> {
        let entropy = Entropy::generate(words_length, rand::random);
        let mnemonics = entropy.to_mnemonics().to_string(&dictionary::ENGLISH);
        Self::recover(&mnemonics, b"iapyx")
    }

    pub fn recover(mnemonics: &str, password: &[u8]) -> Result<Self, Error> {
        Ok(Self {
            inner: Inner::recover(mnemonics, password)
                .map_err(|e| Error::CannotRecover(e.to_string()))?,
        })
    }

    pub fn recover_from_account(secret_key: &[u8]) -> Result<Self, Error> {
        Ok(Self {
            inner: Inner::recover_free_keys(secret_key, [].iter())
                .map_err(|e| Error::CannotRecover(e.to_string()))?,
        })
    }

    pub fn recover_from_utxo(secret_key: &[u8; 64]) -> Result<Self, Error> {
        Ok(Self {
            inner: Inner::recover_free_keys(secret_key, [*secret_key].iter())
                .map_err(|e| Error::CannotRecover(e.to_string()))?,
        })
    }

    pub fn account(&self, discrimination: chain_addr::Discrimination) -> chain_addr::Address {
        self.inner.account(discrimination)
    }

    pub fn id(&self) -> AccountId {
        self.inner.id()
    }

    pub fn retrieve_funds(&mut self, block0_bytes: &[u8]) -> Result<wallet::Settings, Error> {
        self.inner
            .retrieve_funds(block0_bytes)
            .map_err(|e| Error::CannotRetrieveFunds(e.to_string()))
    }

    pub fn convert(&mut self, settings: Settings, valid_until: &BlockDate) -> Conversion {
        self.inner.convert(settings, valid_until)
    }

    pub fn conversion_fragment_ids(
        &mut self,
        settings: Settings,
        valid_until: &BlockDate,
    ) -> Vec<FragmentId> {
        self.convert(settings, valid_until)
            .transactions()
            .iter()
            .map(|x| {
                let fragment = Fragment::deserialize(&mut Codec::new(x.as_slice())).unwrap();
                self.remove_pending_transaction(&fragment.id());
                fragment.id()
            })
            .collect()
    }

    pub fn confirm_all_transactions(&mut self) {
        for id in self.pending_transactions() {
            self.confirm_transaction(id)
        }
    }

    pub fn confirm_transaction(&mut self, id: FragmentId) {
        self.inner.confirm_transaction(id);
    }

    pub fn pending_transactions(&self) -> Vec<FragmentId> {
        self.inner.pending_transactions().into_iter().collect()
    }

    pub fn remove_pending_transaction(&mut self, id: &FragmentId) -> Option<Vec<Input>> {
        self.inner.remove_pending_transaction(id)
    }

    pub fn total_value(&self) -> Value {
        self.inner.total_value()
    }

    pub fn set_state(&mut self, value: Value, counter: u32) {
        self.inner.set_state(value, counter);
    }

    pub fn spending_counter(&self) -> u32 {
        self.inner.spending_counter()
    }

    pub fn vote(
        &mut self,
        settings: Settings,
        proposal: &Proposal,
        choice: Choice,
        valid_until: &BlockDate,
    ) -> Result<Box<[u8]>, Error> {
        self.inner
            .vote(settings, proposal, choice, valid_until)
            .map_err(|e| Error::CannotSendVote(e.to_string()))
    }

    pub fn identifier(&self, discrimination: Discrimination) -> AccountIdentifier {
        let address_readable = match discrimination {
            Discrimination::Test => {
                AddressReadable::from_address("ta", &self.account(discrimination)).to_string()
            }
            Discrimination::Production => {
                AddressReadable::from_address("ca", &self.account(discrimination)).to_string()
            }
        };
        AccountIdentifier::from_str(&address_readable).unwrap()
    }
}

impl std::fmt::Debug for Wallet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.identifier(Discrimination::Production))
    }
}
