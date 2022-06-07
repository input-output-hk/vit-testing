use crate::load::{MultiController, MultiControllerError};
use crate::utils::expiry;
use crate::Wallet;
use chain_impl_mockchain::fragment::FragmentId;
use jormungandr_automation::testing::VoteCastCounter;
use jortestkit::load::{Request, RequestFailure, RequestGenerator};
use rand::seq::SliceRandom;
use rand_core::OsRng;
use std::time::Instant;
use thor::BlockDateGenerator;
use valgrind::SettingsExtensions;
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;
use wallet::Settings;
use wallet_core::Choice;

pub struct WalletRequestGen {
    rand: OsRng,
    multi_controller: MultiController,
    proposals: Vec<FullProposalInfo>,
    options: Vec<u8>,
    wallet_index: usize,
    update_account_before_vote: bool,
    vote_cast_counter: VoteCastCounter,
    block_date_generator: BlockDateGenerator,
    settings: Settings,
}

impl WalletRequestGen {
    pub fn new(
        multi_controller: MultiController,
        update_account_before_vote: bool,
    ) -> Result<Self, super::RequestGenError> {
        let proposals = multi_controller.proposals("dreps")?;
        let vote_plans = multi_controller.backend().vote_plan_statuses()?;
        let settings = multi_controller.backend().settings()?;
        let options = proposals[0]
            .proposal
            .chain_vote_options
            .0
            .values()
            .cloned()
            .collect();

        let vote_cast_counter = VoteCastCounter::new(
            multi_controller.wallet_count(),
            vote_plans
                .iter()
                .map(|v| (v.id.into(), v.proposals.len() as u8))
                .collect(),
        );

        Ok(Self {
            multi_controller,
            proposals,
            options,
            wallet_index: 0,
            update_account_before_vote,
            vote_cast_counter,
            rand: OsRng,
            settings: settings.clone().into_wallet_settings(),
            block_date_generator: expiry::default_block_date_generator(&settings),
        })
    }

    pub fn random_vote(&mut self) -> Result<FragmentId, MultiControllerError> {
        let index = {
            self.wallet_index += 1;
            if self.wallet_index >= self.multi_controller.wallet_count() {
                self.wallet_index = 0;
            }
            self.wallet_index
        };

        // update state of wallet only before first vote.
        // Then relay on mechanism of spending counter auto-update
        if self.update_account_before_vote {
            self.multi_controller
                .update_wallet_state_if(index, &|wallet: &Wallet| {
                    wallet.spending_counter()[0] == 0
                });
        }

        let counter = self.vote_cast_counter.advance_single(index).unwrap();

        let proposal = self
            .proposals
            .get(counter.first().unwrap().first() as usize)
            .unwrap();
        let choice = Choice::new(*self.options.choose(&mut self.rand).unwrap());
        self.multi_controller.vote(
            index,
            proposal,
            choice,
            self.block_date_generator.block_date(),
        )
    }
}

impl RequestGenerator for WalletRequestGen {
    fn split(mut self) -> (Self, Option<Self>) {
        let wallets_len = self.multi_controller.wallets.len();
        if wallets_len <= 1 {
            return (self, None);
        }
        let wallets = self.multi_controller.wallets.split_off(wallets_len / 2);
        let new_gen = Self {
            rand: self.rand,
            multi_controller: MultiController {
                wallets,
                backend: self.multi_controller.backend.clone(),
                settings: self.multi_controller.settings.clone(),
            },
            proposals: self.proposals.clone(),
            options: self.options.clone(),
            wallet_index: 0,
            update_account_before_vote: self.update_account_before_vote,
            vote_cast_counter: self.vote_cast_counter.clone(),
            settings: self.settings.clone(),
            block_date_generator: self.block_date_generator.clone(),
        };

        (self, Some(new_gen))
    }

    fn next(&mut self) -> Result<Request, RequestFailure> {
        let start = Instant::now();
        match self.random_vote() {
            Ok(v) => Ok(Request {
                ids: vec![Some(v.to_string())],
                duration: start.elapsed(),
            }),
            Err(e) => Err(RequestFailure::General(format!("{:?}", e))),
        }
    }
}
