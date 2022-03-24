use crate::qr::PinReadMode;
use crate::qr::QrReader;
use crate::Wallet;
use bech32::FromBase32;
use bip39::Type;
use chain_impl_mockchain::{block::BlockDate, fragment::FragmentId};
use jcli_lib::key::read_bech32;
pub use jormungandr_automation::jormungandr::RestSettings;
use std::iter;
use std::path::Path;
use thiserror::Error;
use valgrind::Proposal;
use valgrind::ProposalExtension;
use valgrind::SettingsExtensions;
use valgrind::ValgrindClient;
use wallet::Settings;
use wallet_core::{Choice, Value};

unsafe impl Send for Wallet {}
use std::convert::TryInto;

pub struct MultiController {
    pub(super) backend: ValgrindClient,
    pub(super) wallets: Vec<Wallet>,
    pub(super) settings: Settings,
}

impl MultiController {
    pub fn generate(
        wallet_backend_address: String,
        words_length: Type,
        count: usize,
        backend_settings: RestSettings,
    ) -> Result<Self, MultiControllerError> {
        let backend = ValgrindClient::new(wallet_backend_address, backend_settings)?;
        let settings = backend.settings()?.into_wallet_settings();
        let wallets = iter::from_fn(|| Some(Wallet::generate(words_length).unwrap()))
            .take(count)
            .collect();
        Ok(Self {
            backend,
            wallets,
            settings,
        })
    }

    pub fn recover(
        wallet_backend_address: &str,
        mnemonics: Vec<String>,
        password: &[u8],
        backend_settings: RestSettings,
    ) -> Result<Self, MultiControllerError> {
        let backend = ValgrindClient::new(wallet_backend_address.to_string(), backend_settings)?;
        let settings = backend.settings()?.into_wallet_settings();
        let wallets = mnemonics
            .iter()
            .map(|x| Wallet::recover(x, password).unwrap())
            .collect();
        Ok(Self {
            backend,
            wallets,
            settings,
        })
    }

    pub fn recover_from_qrs<P: AsRef<Path>>(
        wallet_backend_address: &str,
        qrs: &[P],
        pin_mode: PinReadMode,
        backend_settings: RestSettings,
    ) -> Result<Self, MultiControllerError> {
        let mut backend =
            ValgrindClient::new(wallet_backend_address.to_string(), backend_settings)?;
        let settings = backend.settings()?.into_wallet_settings();

        backend.enable_logs();
        let pin_reader = QrReader::new(pin_mode);
        let secrets = pin_reader.read_qrs(qrs, false);
        let wallets = secrets
            .into_iter()
            .map(|secret| Wallet::recover_from_account(secret.leak_secret().as_ref()).unwrap())
            .collect();

        Ok(Self {
            backend,
            wallets,
            settings,
        })
    }

    pub fn recover_from_sks<P: AsRef<Path>>(
        proxy_address: &str,
        private_keys: &[P],
        backend_settings: RestSettings,
    ) -> Result<Self, MultiControllerError> {
        let backend = ValgrindClient::new(proxy_address.to_string(), backend_settings)?;
        let settings = backend.settings()?.into_wallet_settings();
        let wallets = private_keys
            .iter()
            .map(|x| {
                let (_, data, _) = read_bech32(Some(&x.as_ref().to_path_buf())).unwrap();
                let key_bytes = Vec::<u8>::from_base32(&data).unwrap();
                let data: [u8; 64] = key_bytes.try_into().unwrap();
                Wallet::recover_from_account(&data).unwrap()
            })
            .collect();

        Ok(Self {
            backend,
            wallets,
            settings,
        })
    }

    pub fn proposals(&self) -> Result<Vec<Proposal>, MultiControllerError> {
        self.backend.proposals().map_err(Into::into)
    }

    pub(crate) fn backend(&self) -> &ValgrindClient {
        &self.backend
    }

    pub fn update_wallets_state(&mut self) {
        let backend = self.backend().clone();
        let count = self.wallets.len();
        for (idx, wallet) in self.wallets.iter_mut().enumerate() {
            let account_state = backend.account_state(wallet.id()).unwrap();
            println!("{}/{} Updating account state", idx + 1, count);
            wallet.set_state((*account_state.value()).into(), account_state.counters()[0]);
        }
    }

    pub fn update_wallet_state(&mut self, wallet_index: usize) {
        let backend = self.backend().clone();
        let wallet = self.wallets.get_mut(wallet_index).unwrap();
        let account_state = backend.account_state(wallet.id()).unwrap();
        wallet.set_state((*account_state.value()).into(), account_state.counters()[0]);
    }
    pub fn update_wallet_state_if(
        &mut self,
        wallet_index: usize,
        predicate: &dyn Fn(&Wallet) -> bool,
    ) {
        let wallet = self.wallets.get_mut(wallet_index).unwrap();
        if predicate(wallet) {
            self.update_wallet_state(wallet_index)
        }
    }

    pub fn vote(
        &mut self,
        wallet_index: usize,
        proposal: &Proposal,
        choice: Choice,
        valid_until: BlockDate,
    ) -> Result<FragmentId, MultiControllerError> {
        let wallet = self.wallets.get_mut(wallet_index).unwrap();
        let tx = wallet.vote(
            self.settings.clone(),
            &proposal.clone().into_wallet_proposal(),
            choice,
            &valid_until,
        )?;
        self.backend()
            .send_fragment(tx.to_vec())
            .map_err(Into::into)
    }

    pub fn votes_batch(
        &mut self,
        wallet_index: usize,
        use_v1: bool,
        votes_data: Vec<(&Proposal, Choice)>,
        valid_until: &BlockDate,
    ) -> Result<Vec<FragmentId>, MultiControllerError> {
        let wallet = self.wallets.get_mut(wallet_index).unwrap();
        let account_state = self.backend.account_state(wallet.id())?;

        let mut counter = account_state.counters()[0];
        let settings = self.settings.clone();
        let txs = votes_data
            .into_iter()
            .map(|(p, c)| {
                wallet.set_state((*account_state.value()).into(), counter);
                let tx = wallet
                    .vote(
                        settings.clone(),
                        &p.clone().into_wallet_proposal(),
                        c,
                        valid_until,
                    )
                    .unwrap()
                    .to_vec();
                counter += 1;
                tx
            })
            .rev()
            .collect();

        self.backend()
            .send_fragments_at_once(txs, use_v1)
            .map_err(Into::into)
    }

    pub fn confirm_all_transactions(&mut self) {
        for wallet in self.wallets.iter_mut() {
            wallet.confirm_all_transactions();
        }
    }

    pub fn confirm_transaction(&mut self, fragment_id: FragmentId) {
        for wallet in self.wallets.iter_mut() {
            wallet.confirm_transaction(fragment_id);
        }
    }

    pub fn refresh_wallet(&mut self, wallet_index: usize) -> Result<(), MultiControllerError> {
        let wallet = self.wallets.get_mut(wallet_index).unwrap();
        let account_state = self.backend.account_state(wallet.id())?;
        let value: u64 = (*account_state.value()).into();
        wallet.set_state(Value(value), account_state.counters()[0]);
        Ok(())
    }

    pub fn wallet_count(&self) -> usize {
        self.wallets.len()
    }

    pub fn is_converted(&mut self, wallet_index: usize) -> Result<bool, MultiControllerError> {
        let wallet = self.wallets.get_mut(wallet_index).unwrap();
        self.backend.account_exists(wallet.id()).map_err(Into::into)
    }
}

#[allow(clippy::from_over_into)]
impl Into<Vec<Wallet>> for MultiController {
    fn into(self) -> Vec<Wallet> {
        self.wallets
    }
}

#[derive(Debug, Error)]
pub enum MultiControllerError {
    #[error("wallet error")]
    Wallet(#[from] crate::wallet::Error),
    #[error("wallet error")]
    Backend(#[from] valgrind::Error),
    #[error("controller error")]
    Controller(#[from] crate::ControllerError),
    #[error("pin read error")]
    PinRead(#[from] crate::qr::PinReadError),
    #[error("wallet time error")]
    WalletTime(#[from] wallet::time::Error),
}
