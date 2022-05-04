use std::cmp::Ordering;

use serde::{Deserialize, Serialize};
use vit_servicing_station_lib::{
    db::models::{challenges::Challenge, proposals::FullProposalInfo},
    v0::errors::HandleError,
};
use warp::{Rejection, Reply};

use crate::mode::mock::ContextLock;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Query {
    pub table: Table,
    pub filter: Vec<Constraint>,
    pub order_by: Vec<OrderBy>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Constraint {
    pub search: String,
    pub column: Column,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct OrderBy {
    pub column: Column,
    #[serde(default)]
    pub descending: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Table {
    Challenges,
    Proposals,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Column {
    Title,
    Type,
    Desc,
    Author,
    Funds,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)] // should serialize as if it is either a `Vec<Challenge>` or `Vec<FullProposalInfo>`
pub enum SearchResponse {
    Challenge(Vec<Challenge>),
    Proposal(Vec<FullProposalInfo>),
}

pub async fn search(
    Query {
        table,
        filter,
        order_by,
    }: Query,
    ctx: ContextLock,
) -> Result<(), Rejection> {
    let guard = ctx.lock().unwrap();
    // match table {
    //     Table::Challenges => Ok(SearchResponse::Challenge(search_challenges(
    //         guard.state().vit().challenges(),
    //         filter,
    //         order_by,
    //     )?)),
    //     Table::Proposals => todo!(),
    // };

    Ok(())
}

fn search_challenges(
    mut items: Vec<Challenge>,
    filter: Vec<Constraint>,
    order_by: Vec<OrderBy>,
) -> Result<Vec<Challenge>, Rejection> {
    use Column::*;

    if filter.iter().any(|c| matches!(c.column, Author | Funds)) {
        // TODO: replace with bad request once vit servicing station lib is updated
        return Err(warp::reject::custom(HandleError::NotFound(
            "column not found".to_string(),
        )));
    }

    for Constraint { search, column } in filter {
        let search = search.to_lowercase();
        let pred = |c: &Challenge| -> bool {
            match column {
                Title => c.title.to_lowercase().contains(&search),
                Type => c
                    .challenge_type
                    .to_string()
                    .to_lowercase()
                    .contains(&search),
                Desc => c.description.to_lowercase().contains(&search),
                _ => unreachable!(),
            }
        };

        items = items.into_iter().filter(pred).collect();
    }

    todo!()
}

fn cmp_challenge(c1: &Challenge, c2: &Challenge, order_by: &[OrderBy]) -> Ordering {
    use Column::*;

    match order_by.split_first() {
        None => Ordering::Equal,
        Some((OrderBy { column, descending }, tail)) => {
            let mut ordering = match column {
                Title => c1.title.cmp(&c2.title),
                Type => c1
                    .challenge_type
                    .to_string()
                    .cmp(&c2.challenge_type.to_string()),
                Desc => c1.description.cmp(&c2.description),
                _ => unreachable!(),
            };

            if *descending {
                ordering = ordering.reverse();
            }

            ordering.then_with(|| cmp_challenge(c1, c2, tail))
        }
    }
}

fn cmp_prop(p1: &FullProposalInfo, p2: &FullProposalInfo, order_by: &[OrderBy]) -> Ordering {
    use Column::*;

    match order_by.split_first() {
        None => Ordering::Equal,
        Some((OrderBy { column, descending }, tail)) => {
            let mut ordering = match column {
                Title => p1.proposal.proposal_title.cmp(&p1.proposal.proposal_title),
                Desc => p1
                    .proposal
                    .proposal_summary
                    .cmp(&p2.proposal.proposal_summary),
                Funds => p1.proposal.proposal_funds.cmp(&p2.proposal.proposal_funds),
                Author => p1
                    .proposal
                    .proposer
                    .proposer_name
                    .cmp(&p2.proposal.proposer.proposer_name),
                _ => unreachable!(),
            };

            if *descending {
                ordering = ordering.reverse();
            }

            ordering.then_with(|| cmp_prop(p1, p2, tail))
        }
    }
}

#[cfg(test)]
mod tests {
    use vit_servicing_station_lib::db::models::proposals::test::get_test_proposal;

    use super::*;

    #[test]
    fn can_sort_proposals() {
        let mut prop_1 = get_test_proposal();
        prop_1.proposal.proposal_title = "1".to_string();
        prop_1.proposal.proposal_summary = "1".to_string();

        let prop_2 = {
            let mut temp = prop_1.clone();
            temp.proposal.proposal_title = "1".to_string();
            temp.proposal.proposal_summary = "2".to_string();
            temp
        };

        let prop_3 = {
            let mut temp = prop_1.clone();
            temp.proposal.proposal_title = "2".to_string();
            temp.proposal.proposal_summary = "1".to_string();
            temp
        };

        let mut vec = vec![&prop_3, &prop_2, &prop_1];
        use Column::*;

        let order_by = vec![
            OrderBy {
                column: Title,
                descending: false,
            },
            OrderBy {
                column: Desc,
                descending: false,
            },
        ];

        vec.sort_by(|a, b| cmp_prop(a, b, &order_by));

        assert_eq!(vec, vec![&prop_1, &prop_2, &prop_3])
    }
}
