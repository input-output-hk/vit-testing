use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FundInfo {
    pub goals: Vec<String>,
    pub results_url: String,
    pub survey_url: String,
    pub fund_name: String,
    pub fund_id: i32,
}

impl From<i32> for FundInfo {
    fn from(fund_id: i32) -> Self {
        Self {
            results_url: "https://catalyst.domain/result".to_string(),
            survey_url: "https://catalyst.domain/survey".to_string(),
            goals: vec![
                "first Goal".to_string(),
                "second Goal".to_string(),
                "third Goal".to_string(),
            ],
            fund_id,
            fund_name: format!("Fund{}", fund_id),
        }
    }
}
