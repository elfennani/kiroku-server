use crate::domain::models;
use graphql_client::GraphQLQuery;
use rusqlite::types::Value;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;
use crate::errors::AppError;

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
    query_path = "schemes/anilist/queries/ongoing.graphql",
    response_derives = "Debug,Serialize"
)]
pub struct OngoingQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemes/anilist/schema.json",
    query_path = "schemes/anilist/queries/media_details.graphql",
    response_derives = "Debug,Serialize"
)]
pub struct MediaDetailsQuery;

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

/// graphql_client creates a new rust enum for every query that the same graphql enum.
/// Serializing to a string then mapping to a Rust enum is a better option than creating
/// the same mapper for every generated query.
impl From<String> for models::Status {
    fn from(value: String) -> Self {
        match value.as_str() {
            "CURRENT" => Self::Current,
            "PLANNING" => Self::Planned,
            "COMPLETED" => Self::Completed,
            "DROPPED" => Self::Dropped,
            "PAUSED" => Self::Paused,
            "REPEATING" => Self::Revisiting,
            _ => Self::Unknown(value),
        }
    }
}

impl TryFrom<serde_json::Value> for models::Status {
    type Error = AppError;

    fn try_from(value: serde_json::Value) -> Result<Self, AppError> {
        if value.is_string(){
            Ok(Self::from(value.as_str().unwrap().to_string()))
        } else {
            Err(AppError::BadRequest("Failed to parse media status".to_string()))
        }
    }
}
