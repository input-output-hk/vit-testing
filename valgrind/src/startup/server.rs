use crate::startup::{Certs, Protocol};
use std::net::SocketAddr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Malformed proxy address: {0}")]
    Proxy(String),
    #[error("Malformed vit address: {0}")]
    VitStation(String),
    #[error("Malformed node rest address: {0}")]
    NodeRest(String),
}

pub struct ProxyServerStub {
    protocol: Protocol,
    address: String,
    vit_address: String,
    node_rest_address: String,
    block0: Vec<u8>,
}

impl ProxyServerStub {
    pub fn new_https(
        certs: Certs,
        address: String,
        vit_address: String,
        node_rest_address: String,
        block0: Vec<u8>,
    ) -> Self {
        Self::new(
            certs.into(),
            address,
            vit_address,
            node_rest_address,
            block0,
        )
    }

    pub fn new_http(
        address: String,
        vit_address: String,
        node_rest_address: String,
        block0: Vec<u8>,
    ) -> Self {
        Self::new(
            Default::default(),
            address,
            vit_address,
            node_rest_address,
            block0,
        )
    }

    pub fn protocol(&self) -> &Protocol {
        &self.protocol
    }

    pub fn new(
        protocol: Protocol,
        address: String,
        vit_address: String,
        node_rest_address: String,
        block0: Vec<u8>,
    ) -> Self {
        Self {
            protocol,
            address,
            vit_address,
            node_rest_address,
            block0,
        }
    }

    pub fn block0(&self) -> Vec<u8> {
        self.block0.clone()
    }

    pub fn address(&self) -> String {
        self.address.parse().unwrap()
    }

    pub fn vit_address(&self) -> String {
        self.vit_address.parse().unwrap()
    }

    pub fn node_rest_address(&self) -> String {
        self.node_rest_address.parse().unwrap()
    }

    pub fn base_address(&self) -> SocketAddr {
        self.address.parse().unwrap()
    }

    pub fn http_vit_address(&self) -> String {
        format!("http://{}/", self.vit_address)
    }

    pub fn http_node_address(&self) -> String {
        format!("http://{}/", self.node_rest_address)
    }
}
