use crate::scenario::{
    settings::PrepareWalletProxySettings, vit_station::VitStationSettings, wallet::NodeAlias,
};
use jormungandr_scenario_tests::Context;
pub use jormungandr_testing_utils::testing::network::WalletProxySettings;
use rand::CryptoRng;
use rand::RngCore;
use std::collections::HashMap;

impl PrepareWalletProxySettings for WalletProxySettings {
    fn prepare<RNG>(
        _context: &mut Context<RNG>,
        vit_stations: &HashMap<NodeAlias, VitStationSettings>,
    ) -> Self
    where
        RNG: RngCore + CryptoRng,
    {
        let vit_station_settings = vit_stations
            .values()
            .next()
            .expect("no vit stations defined");

        WalletProxySettings {
            proxy_address: "127.0.0.1:8080".parse().unwrap(),
            vit_station_address: vit_station_settings.address,
            node_backend_address: None,
        }
    }
}
