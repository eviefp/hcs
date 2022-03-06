mod args;
mod cal;
mod common;
mod insert;
mod next;
mod day;

use args::{Args, Commands};
use chrono::TimeZone;
use chrono_tz::Europe::Bucharest;
use clap::Parser;
use next::perform_next_event_query;
use owo_colors::{OwoColorize, Stream::Stdout};
use std::error::Error;
use day::perform_day_event_query;

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

    println!(
        "Removed {} events!",
        del.if_supports_color(Stdout, |t| t.red())
    );
    println!(
        "Inserted {} events!",
        ins.if_supports_color(Stdout, |t| t.green())
    );

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

fn print_event(e: day::today_event_query::TodayEventQueryEvents) -> Result<(), Box<dyn Error>> {
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
    println!(
        "{} {}",
        start_fmt.if_supports_color(Stdout, |t| t.green()),
        e.summary
            .unwrap()
            .if_supports_color(Stdout, |t| t.magenta())
    );
    Ok(())
}

fn print_next_event(
    o: Option<next::next_event_query::NextEventQueryEvents>,
    xmobar: bool,
) -> Result<(), Box<dyn Error>> {
    // TODO: figure out why local_offset doesn't work
    match o {
        None => {
            if xmobar {
                println!("<fc=#00ed8a>N/A</fc>");
            } else {
                println!(
                    "{}",
                    "No events today.".if_supports_color(Stdout, |t| t.green())
                );
            }
        }
        Some(e) => {
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
            if xmobar {
                let mut out = e.summary.unwrap();
                let len = out.len();
                out.truncate(16);
                if len > out.len() {
                    out = out + "...";
                }
                println!("<fc=#00ed8a>{}</fc> <fc=#00d9ed>{}</fc>", start_fmt, out);
            } else {
                println!(
                    "{} {}",
                    start_fmt.if_supports_color(Stdout, |t| t.green()),
                    e.summary
                        .unwrap()
                        .if_supports_color(Stdout, |t| t.magenta())
                );
            }
        }
    }
    Ok(())
}

async fn day(
    hasura: common::Hasura,
    day: chrono::DateTime<chrono::Local>,
) -> Result<(), Box<dyn Error>> {
    let result = perform_day_event_query(hasura, day).await?;
    result
        .into_iter()
        .map(print_event)
        .collect::<Result<Vec<()>, _>>()?;

    Ok(())
}

async fn today(hasura: common::Hasura) -> Result<(), Box<dyn Error>> {
    let today = chrono::Local::today()
        .and_hms_opt(0, 0, 0)
        .ok_or(HcsError::TodayError {})?;
    day(hasura, today).await
}

async fn tomorrow(hasura: common::Hasura) -> Result<(), Box<dyn Error>> {
    let tomorrow = chrono::Local::today()
        .and_hms_opt(0, 0, 0)
        .ok_or(HcsError::TomorrowError {})?
        .checked_add_signed(chrono::Duration::days(1))
        .ok_or(HcsError::TomorrowError {})?;
    day(hasura, tomorrow).await
}

async fn next(hasura: common::Hasura, xmobar: bool) -> Result<(), Box<dyn Error>> {
    let result = perform_next_event_query(hasura).await?;
    print_next_event(result, xmobar)?;
    Ok(())
}

fn load_config() -> Result<common::Config, Box<dyn Error>> {
    let hcs = xdg::BaseDirectories::with_prefix("hcs")?;
    let config_path = hcs.place_config_file("hcs.toml").expect("");
    let config_content = std::fs::read_to_string(config_path)?;
    let config: common::Config = toml::from_str(&config_content)?;
    Ok(config)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config: common::Config = load_config()?;
    match Args::parse().command {
        Commands::Import { target } => import(config, target).await?,
        Commands::Today {} => today(config.hasura).await?,
        Commands::Tomorrow {} => tomorrow(config.hasura).await?,
        Commands::Next { xmobar } => next(config.hasura, xmobar).await?,
    };

    Ok(())
}
