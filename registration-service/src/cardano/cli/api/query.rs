use crate::cardano::cli::command::QueryCommand;
use crate::cardano::error::Error;
use crate::config::NetworkType;
use jortestkit::prelude::ProcessOutput;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;

pub struct Query {
    query_command: QueryCommand,
}

impl Query {
    pub fn new(query_command: QueryCommand) -> Self {
        Self { query_command }
    }

    pub fn tip(self, network: NetworkType) -> Result<String, Error> {
        let mut command = self.query_command.tip(network).build();
        let content = command
            .output()
            .map_err(|x| Error::CannotGetOutputFromCommand(x.to_string()))?
            .as_lossy_string();

        println!("raw output of query tip: {}", content);
        Ok(content)
    }

    pub fn utxo<S: Into<String>>(
        self,
        payment_address: S,
        network: NetworkType,
    ) -> Result<String, Error> {
        let mut command = self.query_command.utxo(network, payment_address).build();
        let content = command
            .output()
            .map_err(|x| Error::CannotGetOutputFromCommand(x.to_string()))?
            .as_lossy_string();

        println!("raw output of query utxo: {}", content);
        Ok(content)
    }

    pub fn funds<S: Into<String>>(
        self,
        payment_address: S,
        network: NetworkType,
    ) -> Result<u64, Error> {
        get_funds(self.utxo(payment_address, network)?).map_err(Into::into)
    }
}

/// Supported output
/// {
///     "bf810bff3f979e28441775077524d41741542d1f237b0b7d6a698164dcded29b#0": {
///         "address": "addr_test1vqtsh379yx9hmn8ypedzx270q9ueea24suf3zu85p2mv2sgra46ef",
///         "value": {
///             "lovelace": 9643918
///         }
///     }
/// }
pub fn get_funds(output: String) -> Result<u64, Error> {
    println!("get_funds: {}", output);
    let response: FundsResponse =
        serde_json::from_str(&output).map_err(|_| Error::CannotParseCardanoCliOutput(output))?;
    Ok(response.get_first()?.value.lovelace)
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct FundsResponse {
    #[serde(flatten)]
    content: HashMap<String, FundsEntry>,
}

impl FundsResponse {
    pub fn get_first(&self) -> Result<FundsEntry, Error> {
        Ok(self
            .content
            .iter()
            .next()
            .ok_or_else(|| Error::CannotParseCardanoCliOutput("empty response".to_string()))?
            .1
            .clone())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct FundsEntry {
    address: String,
    value: FundsValue,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct FundsValue {
    lovelace: u64,
}

/// Supported output:
/// Vote public key used        (hex): c6b6d184ea26781f00b9034ec0ba974f2f833788ce2e24cc37e9e8f41131e1fa
/// Stake public key used       (hex): e542b6a0ced80e1ab5bda70311bf643b9011ee04411737f3e0136825ef47f2d8
/// Rewards address used        (hex): 60170bc7c5218b7dcce40e5a232bcf01799cf55587131170f40ab6c541
/// Slot registered:                   25398498
/// Vote registration signature (hex): e5cc2e1a9344794cbad76bb65d485776aa560baca6133cdfe77827b15dd0e4c883c32e7177dc15d55e34f79df7ffaebca4d271271c6615b0dacc90e36fb22f03
pub fn get_slot_no(output: Vec<String>) -> Result<u64, Error> {
    output
        .iter()
        .find(|x| x.contains("Slot registered"))
        .ok_or_else(|| Error::CannotParseVoterRegistrationOutput(output.clone()))?
        .split_whitespace()
        .nth(2)
        .ok_or_else(|| Error::CannotParseVoterRegistrationOutput(output.clone()))?
        .parse()
        .map_err(|_| Error::CannotParseVoterRegistrationOutput(output.clone()))
}

#[cfg(test)]
mod tests {

    use super::{get_funds, get_slot_no};

    #[test]
    pub fn test_slot_no_extraction() {
        let content = vec![
            "Vote public key used        (hex): c6b6d184ea26781f00b9034ec0ba974f2f833788ce2e24cc37e9e8f41131e1fa".to_string(),
            "Stake public key used       (hex): e542b6a0ced80e1ab5bda70311bf643b9011ee04411737f3e0136825ef47f2d8".to_string(),
            "Rewards address used        (hex): 60170bc7c5218b7dcce40e5a232bcf01799cf55587131170f40ab6c541".to_string(),
            "Slot registered:                   25398498".to_string(),
            "Vote registration signature (hex): e5cc2e1a9344794cbad76bb65d485776aa560baca6133cdfe77827b15dd0e4c883c32e7177dc15d55e34f79df7ffaebca4d271271c6615b0dacc90e36fb22f03".to_string()
        ];

        assert_eq!(get_slot_no(content).unwrap(), 25398498);
    }

    #[test]
    pub fn test_funds_extraction() {
        let content = vec![
        "{", 
        "    \"bf810bff3f979e28441775077524d41741542d1f237b0b7d6a698164dcded29b#0\": {",
        "        \"address\": \"addr_test1vqtsh379yx9hmn8ypedzx270q9ueea24suf3zu85p2mv2sgra46ef\",",
        "        \"value\": {", 
        "            \"lovelace\": 9643918", 
        "        }", 
        "    }", 
        "}"];
        assert_eq!(get_funds(content.join(" ")).unwrap(), 9643918);
    }
}
