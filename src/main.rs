mod args;
mod cal;
mod common;
mod insert;
mod next;
mod today;

use args::{Args, Commands};
use chrono::TimeZone;
use chrono_tz::Europe::Bucharest;
use clap::Parser;
use next::perform_next_event_query;
use owo_colors::OwoColorize;
use std::error::Error;
use today::perform_today_event_query;

use crate::common::HcsError;

async fn import(config: common::Config, target: String) -> Result<(), Box<dyn Error>> {
    let import = find_import(config.import, target.clone())?;
    let ical = cal::grab_ical(import.url).await?;
    let events: Vec<insert::insert_event_query::events_insert_input> = ical
        .events
        .into_iter()
        .map(|e| insert::ical_event_to_event(target.clone(), e))
        .collect();

    let (del, ins) = insert::perform_insert_event(config.hasura, import.name, events).await?;

    println!("Removed {} events!", del.red());
    println!("Inserted {} events!", ins.green());

    Ok(())
}

fn find_import(
    import: Vec<common::Import>,
    target: String,
) -> Result<common::Import, Box<dyn Error>> {
    import
        .into_iter()
        .find(|i| i.name == target)
        .ok_or(common::HcsError::ImportMissingKey {}.into())
}

fn print_event(e: today::today_event_query::TodayEventQueryEvents) -> Result<(), Box<dyn Error>> {
    // TODO: figure out why local_offset doesn't work
    let local_tz = Some(Bucharest).ok_or(HcsError::MissingStart {})?;
    let start = chrono_tz::UTC
        .timestamp_opt(
            e.start
                .ok_or(HcsError::MissingStart {})?
                .to_unix_timestamp_ms()
                / 1000,
            0,
        )
        .single()
        .ok_or(HcsError::MissingStart {})?
        .with_timezone(&local_tz);
    let start_fmt = start.format("%H:%M").to_string();
    println!("{} {}", start_fmt.green(), e.summary.unwrap().magenta());
    Ok(())
}

fn print_next_event(e: next::next_event_query::NextEventQueryEvents) -> Result<(), Box<dyn Error>> {
    // TODO: figure out why local_offset doesn't work
    let local_tz = Some(Bucharest).ok_or(HcsError::MissingStart {})?;
    let start = chrono_tz::UTC
        .timestamp_opt(
            e.start
                .ok_or(HcsError::MissingStart {})?
                .to_unix_timestamp_ms()
                / 1000,
            0,
        )
        .single()
        .ok_or(HcsError::MissingStart {})?
        .with_timezone(&local_tz);
    let start_fmt = start.format("%H:%M").to_string();
    println!("{} {}", start_fmt.green(), e.summary.unwrap().magenta());
    Ok(())
}

async fn today(hasura: common::Hasura) -> Result<(), Box<dyn Error>> {
    let result = perform_today_event_query(hasura).await?;
    result
        .into_iter()
        .map(print_event)
        .collect::<Result<Vec<()>, _>>()?;

    Ok(())
}

async fn next(hasura: common::Hasura) -> Result<(), Box<dyn Error>> {
    let result = perform_next_event_query(hasura).await?;
    print_next_event(result)?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config_content = std::fs::read_to_string("hcs.toml")?;
    let config: common::Config = toml::from_str(&config_content)?;
    match Args::parse().command {
        Commands::Import { target } => import(config, target).await?,
        Commands::Today {} => today(config.hasura).await?,
        Commands::Next {} => next(config.hasura).await?,
    };

    Ok(())
}
