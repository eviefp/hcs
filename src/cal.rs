use ical::parser::ical::component::IcalCalendar;
use reqwest;
use std::error::Error;
use std::io::BufReader;

use crate::common::HcsError;

pub async fn grab_ical(url: String) -> Result<IcalCalendar, Box<dyn Error>> {
    let parsed_url = reqwest::Url::parse(url.as_str())?;
    let ical = reqwest::get(parsed_url).await?.text().await?;
    let bytes = ical.into_bytes();
    let buf = BufReader::new(bytes.as_slice());
    let mut reader = ical::IcalParser::new(buf);
    match reader.next() {
        None => Err(HcsError::CalError {}.into()),
        Some(cal) => match cal {
            Err(_) => Err(HcsError::CalError {}.into()),
            Ok(c) => Ok(c),
        },
    }
}
