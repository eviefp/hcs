use chrono::offset::TimeZone;
use chrono::NaiveDateTime;
use chrono_tz::Tz;
use chrono_tz::UTC;
use graphql_client::{GraphQLQuery, Response};
use iso8061_timestamp::Timestamp;
use reqwest;
use std::error::Error;
use std::str::FromStr;

use crate::common::{timestamptz, HcsError};

// Insert events
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "gql/schema.graphql",
    query_path = "gql/events_insert.graphql",
    response_derives = "Debug"
)]
pub struct InsertEventQuery;

pub async fn perform_insert_event(
    hasura: crate::common::Hasura,
    key: String,
    obj: Vec<insert_event_query::events_insert_input>,
) -> Result<(i64, i64), Box<dyn Error>> {
    let no_vars = insert_event_query::Variables {
        objects: obj,
        key: Some(key),
    };
    let request_body = InsertEventQuery::build_query(no_vars);
    let client = reqwest::Client::new();
    let res = client
        .post(hasura.url)
        .header("x-hasura-admin-secret", hasura.admin_key)
        .json(&request_body)
        .send()
        .await?;
    let response_body: Response<insert_event_query::ResponseData> = res.json().await?;
    response_body
        .data
        .and_then(|d| {
            let ins = d.insert_events?.affected_rows;
            let del = d.delete_events?.affected_rows;
            Some((del, ins))
        })
        .ok_or(HcsError::InsertDataError {}.into())
}

pub fn ical_event_to_event(
    key: String,
    ev: ical::parser::ical::component::IcalEvent,
) -> insert_event_query::events_insert_input {
    let mut ret = insert_event_query::events_insert_input {
        attach: None,
        calendar_uid: None,
        created_at: None,
        description: None,
        key: Some(key),
        end: None,
        location: None,
        organizer: None,
        start: None,
        status: None,
        summary: None,
        updated_at: None,
    };
    ev.properties
        .into_iter()
        .for_each(|p| match p.name.as_str() {
            "ATTACH" => ret.attach = p.value,
            "UID" => ret.calendar_uid = p.value,
            "CREATED" => ret.created_at = p.value.and_then(|s| Timestamp::parse(s.as_str())),
            "DESCRIPTION" => ret.description = p.value,
            "DTEND" => ret.end = p.value.and_then(|s| Timestamp::parse(s.as_str())),
            "LOCATION" => ret.location = p.value,
            "ORGANIZER" => ret.organizer = p.value,
            "DTSTART" => ret.start = parse_time(p),
            "STATUS" => ret.organizer = p.value,
            "SUMMARY" => ret.summary = p.value,
            "LAST-MODIFIED" => ret.updated_at = p.value.and_then(|s| Timestamp::parse(s.as_str())),
            _ => {}
        });
    ret
}

fn parse_time(p: ical::property::Property) -> Option<Timestamp> {
    // Assumption is, if there's params, there's a single param which is either "TZID" or "VALUE".
    match p.params.and_then(|x| x.into_iter().next()) {
        // Param key can either be "TZID" or "VALUE"
        Some((key, values)) => match key.as_str() {
            // If it's "TZID", then there's a single timezone in the values.
            // TODO: There's got to be a better way.
            "TZID" => match values.first().and_then(|tz| Tz::from_str(tz.as_str()).ok()) {
                Some(tz) => {
                    // Trick the parser to accept this as UTC.
                    let val = p.value? + "Z";
                    // Parse it as UTC.
                    let parse = Timestamp::parse(&val)?;
                    // Grab the timestamp as 'Naive', meaning with no associated timezone info.
                    let dt =
                        NaiveDateTime::from_timestamp_opt(parse.to_unix_timestamp_ms() / 1_000, 0)?;
                    // Parse it as a local time in the 'tz' timezone, then convert to UTC.
                    let ld = tz.from_local_datetime(&dt).single()?.with_timezone(&UTC);
                    // And finally, go back to the initial format.
                    Some(Timestamp::from_unix_timestamp(ld.timestamp()))
                }
                None => None,
            },

            "VALUE" => match values.first().as_deref().map(String::as_str) {
                Some("DATE") => {
                    // Trick the parser to accept this as UTC.
                    let val = p.value? + "T000000Z";
                    Timestamp::parse(&val)
                }
                // TODO: handle this case
                _ => None,
            },
            // TODO: this should probably be changed to return a Result and we should be
            // returning the context of the error here for future debugging.
            _ => None,
        },
        _ => p.value.and_then(|s| Timestamp::parse(s.as_str())),
    }
}
