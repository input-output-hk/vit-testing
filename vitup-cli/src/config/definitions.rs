use crate::utils::version::VersionReq;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Config(Vec<Environment>);

impl Config {
    pub fn get_environment(&self, name: &str) -> Option<&Environment> {
        self.0.iter().find(|environment| environment.name() == name)
    }

    pub fn environments(&self) -> &[Environment] {
        &self.0
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Environment {
    name: String,
    description: String,
    endpoint: String,
    token: Option<String>,
}

impl Environment {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn endpoint(&self) -> Option<String> {
        self.token.clone()
    }
}