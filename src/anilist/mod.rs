use std::fmt::Display;
use crate::anilist::ongoing_query::MediaListStatus;
use graphql_client::GraphQLQuery;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::str::FromStr;

pub mod client;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemes/anilist/schema.json",
    query_path = "schemes/anilist/queries/viewer.graphql",
    response_derives = "Debug"
)]
pub struct ViewerQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemes/anilist/schema.json",
    query_path = "schemes/anilist/queries/ongoing_query.graphql",
    response_derives = "Debug"
)]
pub struct OngoingQuery;

#[derive(Deserialize, Serialize, Debug)]
pub struct GraphQLResponse<T>(pub graphql_client::Response<T>);

impl<T> Deref for GraphQLResponse<T> {
    type Target = graphql_client::Response<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> GraphQLResponse<T> {
    pub fn into_inner(self) -> graphql_client::Response<T> {
        self.0
    }
}

impl Display for MediaListStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            MediaListStatus::CURRENT => String::from_str("CURRENT").unwrap(),
            MediaListStatus::PLANNING => String::from_str("PLANNING").unwrap(),
            MediaListStatus::COMPLETED => String::from_str("COMPLETED").unwrap(),
            MediaListStatus::DROPPED => String::from_str("DROPPED").unwrap(),
            MediaListStatus::PAUSED => String::from_str("PAUSED").unwrap(),
            MediaListStatus::REPEATING => String::from_str("REPEATING").unwrap(),
            MediaListStatus::Other(_) => "OTHER".to_string(),
        };
        write!(f, "{}", str)
    }
}
