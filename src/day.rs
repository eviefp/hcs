use graphql_client::GraphQLQuery;
use iso8061_timestamp::Timestamp;
use reqwest;
use std::error::Error;

use crate::common::HcsError;
use crate::common::timestamptz;

// Today's events
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "gql/schema.graphql",
    query_path = "gql/events_today.graphql",
    response_derives = "Debug"
)]
pub struct TodayEventQuery;

pub async fn perform_day_event_query(
    hasura: crate::common::Hasura,
    today: chrono::DateTime<chrono::Local>,
) -> Result<Vec<today_event_query::TodayEventQueryEvents>, Box<dyn Error>> {
    let tomorrow = today
        .checked_add_signed(chrono::Duration::days(1))
        .ok_or(HcsError::TodayError {})?;
    let start: Timestamp = Timestamp::from_unix_timestamp(today.timestamp());
    let end: Timestamp = Timestamp::from_unix_timestamp(tomorrow.timestamp());
    let no_vars = today_event_query::Variables {
        start: Some(start),
        end: Some(end),
    };
    let request_body = TodayEventQuery::build_query(no_vars);
    let client = reqwest::Client::new();
    let res = client
        .post(hasura.url)
        .header("x-hasura-admin-secret", hasura.admin_key)
        .json(&request_body)
        .send()
        .await?;
    let response_body: graphql_client::Response<today_event_query::ResponseData> =
        res.json().await?;
    response_body
        .data
        .ok_or(HcsError::TodayError {}.into())
        .map(|x| x.events)
}
