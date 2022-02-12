use graphql_client::GraphQLQuery;
use iso8061_timestamp::Timestamp;
use reqwest;
use std::error::Error;

use crate::common::timestamptz;
use crate::common::HcsError;

// Next event
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "gql/schema.graphql",
    query_path = "gql/events_next.graphql",
    response_derives = "Debug"
)]
pub struct NextEventQuery;

pub async fn perform_next_event_query(
    hasura: crate::common::Hasura,
) -> Result<Option<next_event_query::NextEventQueryEvents>, Box<dyn Error>> {
    let today = chrono::Local::now();
    let tomorrow = today
        .checked_add_signed(chrono::Duration::days(1))
        .ok_or(HcsError::NextError {})?;
    let start: Timestamp = Timestamp::from_unix_timestamp(today.timestamp());
    let end: Timestamp = Timestamp::from_unix_timestamp(tomorrow.timestamp());
    let no_vars = next_event_query::Variables {
        start: Some(start),
        end: Some(end),
    };
    let request_body = NextEventQuery::build_query(no_vars);
    let client = reqwest::Client::new();
    let res = client
        .post(hasura.url)
        .header("x-hasura-admin-secret", hasura.admin_key)
        .json(&request_body)
        .send()
        .await?;
    let response_body: graphql_client::Response<next_event_query::ResponseData> =
        res.json().await?;
    response_body
        .data
        .ok_or(HcsError::NextError {}.into())
        .map(|x| x.events.into_iter().next())
}
