//! # Solar API
//! Rust library for accessing the Solar Edge API. This library uses the API documentation found [here](https://knowledge-center.solaredge.com/en/search?search=api&sort_by=search_api_relevance)
//!
//! # API Key and Site ID
//! To access the data of your installation, you need to get an API key. You can get this from the SolardEdge Monitoring Portal. Log in with your SolarEdge Account, go to the Admin section, Site Access tab and activate API access. Mark the checkbox and you will see the API Key and Site ID
//!
//! # Rate limited
//! Please be aware that the API is rate limited, i.e. it will block requests after reaching a maximum of requests in an hour. It will be available again after that hour. Also note that the measurements seem to be limited to one per fifteen minutes. You can consider scheduling a read of data ±15 minutes after the timestamp of last read measurement. For example you can use a duration of 15m 10s:
//!
//! ```rust
//! let next_update = last_updated_datetime + Duration::seconds(15 * 60 + 10);
//! ```
//!
//! There is a convenience method to help with this:
//!
//! ```rust
//! let site_overview: Overview = overview(api_key, site_id);
//! let (next_update, duration_from_now) = site_overview.estimated_next_update();
//! ```
//!
//! Please note that sometimes the API is a bit later. The `duration_from_now` can be negative then and you have to wait a bit more like in the example below. Please note that `checked_add` is needed here to handle adding negative `duration_from_now`.
//!
//! ```rust
//! let site_overview: Overview = overview(api_key, site_id);
//! let (next_update, duration_from_now) = site_overview.estimated_next_update();
//!
//! let next = Instant::now()
//!     .checked_add(Duration::from_secs(duration_from_now as u64))
//!     .unwrap_or(Instant::now() + Duration::from_secs(30));
//!
//! // wait next or set timeout at next_update before 
//! // getting power or energy data
// ```

mod site;

use chrono::NaiveDateTime;
use log::{debug, trace};
use reqwest::StatusCode;
use std::collections::HashMap;
use thiserror::Error;

pub use site::{
    DataPeriod, GeneratedEnergy, GeneratedEnergyValue, GeneratedPower, GeneratedPowerPerTimeUnit,
    GeneratedPowerValue, Location, Overview, PrimaryModule, PublicSettings, Site, TimeData,
    TimeUnit,
};

/// Possible errors that this lib can return. The underlying errors are included,
/// either being [`request::Error``] or [`serde_json::Error`]
#[derive(Error, Debug)]
pub enum SolarApiError {
    #[error("Could not retrieve data from SolarEdge Monitoring API")]
    NetworkError(reqwest::Error),
    #[error("API returned an Error")]
    ApiError(reqwest::Error),
    #[error("Not allowed to access API. Is the site id valid? Is your API token valid?")]
    ForbiddenError(reqwest::Error),
    #[error("Could not parse result from SolardEdge monitoring api")]
    ParseError(#[from] serde_json::Error),
}

impl From<reqwest::Error> for SolarApiError {
    fn from(error: reqwest::Error) -> Self {
        if let Some(status) = error.status() {
            if status.is_client_error() || status.is_server_error() {
                if status == StatusCode::from_u16(403).unwrap() {
                    return SolarApiError::ForbiddenError(error);
                }
                return SolarApiError::ApiError(error);
            }
        }
        SolarApiError::NetworkError(error)
    }
}

const BASE_URL: &str = "monitoringapi.solaredge.com";

fn default_map(api_key: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("api_key".into(), api_key.into());
    map
}

fn map_to_params(map: &HashMap<String, String>) -> String {
    let mut params = map
        .iter()
        .fold(String::new(), |s, (k, v)| s + &format!("{}={}&", k, v));

    // remove trailing &
    params.pop();
    params
}

fn to_url(path: &str, params: &HashMap<String, String>) -> String {
    let params = map_to_params(params);
    let url = format!("https://{}{}?{}", BASE_URL, path, params);
    url
}

fn call_url(url: &str) -> Result<String, reqwest::Error> {
    trace!("Calling {}", url);
    let reply = reqwest::blocking::get(url)?.error_for_status()?;

    trace!("reply: {:?}", reply);
    let reply_text = reply.text()?;
    trace!("reply text: {}", reply_text);
    Ok(reply_text)
}

/// List all sites of customer. Each [`Site`] has an id that can be
/// used to retrieve detailled information using for example [`energy`]
pub fn list(api_key: &str) -> Result<Vec<site::Site>, SolarApiError> {
    debug!("Calling list of sites");
    let map = default_map(api_key);
    let url = to_url("/sites/list", &map);
    let reply_text = call_url(&url)?;

    trace!("Parsing");
    let reply: site::SitesReply = serde_json::from_str(&reply_text)?;

    Ok((*reply.sites()).clone())
}

/// Displays the site details, such as name, location, status, etc.
pub fn details(api_key: &str, site_id: u32) -> Result<site::Site, SolarApiError> {
    debug!("Getting details of {site_id}");
    let params = default_map(api_key);
    let path = format!("/site/{site_id}/details");
    let url = to_url(&path, &params);
    let reply_text = call_url(&url)?;

    trace!("Parsing json");
    let site: site::SiteDetails = serde_json::from_str(&reply_text)?;

    Ok(site.details)
}

/// Return the energy production start and end dates of the site
pub fn data_period(api_key: &str, site_id: u32) -> Result<site::DataPeriod, SolarApiError> {
    debug!("Getting data_period of {site_id}");
    let params = default_map(api_key);
    let path = format!("/site/{site_id}/dataPeriod");
    let url = to_url(&path, &params);
    let reply_text = call_url(&url)?;

    trace!("Parsing json");
    let period: site::DataPeriodReply = serde_json::from_str(&reply_text)?;

    Ok(period.data_period)
}

/// Display the site overview data.
pub fn overview(api_key: &str, site_id: u32) -> Result<site::Overview, SolarApiError> {
    debug!("Getting overview of {}", site_id);
    let params = default_map(api_key);
    let path = format!("/site/{}/overview", site_id);
    let url = to_url(&path, &params);
    let reply_text = call_url(&url)?;

    trace!("Parsing json");
    let overview: site::OverviewReply = serde_json::from_str(&reply_text)?;

    Ok(overview.overview)
}

/// Return the site energy measurements. Usage limitation: This API is limited
/// to one year when using `time_unit=`[`TimeUnit::Day`] (i.e., daily resolution)
/// and to one month when using `time_unit=`[`TimeUnit::QuarterOfAnHour`] or
/// `time_unit=`[`TimeUnit::Hour`]`. This means that the period between
/// `period.end_time` and `period.start_time` should not exceed one year or one
/// month respectively. If the period is longer, the system will generate error
pub fn energy(
    api_key: &str,
    site_id: u32,
    period: DataPeriod,
    time_unit: TimeUnit,
) -> Result<site::GeneratedEnergy, SolarApiError> {
    debug!(
        "Getting energy for {}-{} with unit {}",
        period.start_date,
        period.end_date,
        time_unit.to_param()
    );

    let mut params = default_map(api_key);
    params.insert("startDate".into(), period.formatted_start_date());
    params.insert("endDate".into(), period.formatted_end_date());
    params.insert("timeUnit".into(), time_unit.to_param().into());
    let path = format!("/site/{site_id}/energy");
    let url = to_url(&path, &params);
    let reply_text = call_url(&url)?;

    trace!("Parsing json");
    let energy: site::GeneratedEnergyReply = serde_json::from_str(&reply_text)?;

    Ok(energy.energy)
}

/// Return the site power measurements in 15 minutes resolution. This API is 
/// limited to one-month period. This means that the period between `end_datetime`
/// and `start_datetime` should not exceed one month. If the period is longer, 
/// the system will generate error .
pub fn power(
    api_key: &str,
    site_id: u32,
    start_datetime: NaiveDateTime,
    end_datetime: NaiveDateTime,
) -> Result<site::GeneratedPowerPerTimeUnit, SolarApiError> {
    debug!("Getting power for {}-{}", start_datetime, end_datetime,);

    let mut params = default_map(api_key);
    params.insert(
        "startTime".into(),
        format!("{}", start_datetime.format("%Y-%m-%d %H:%M:%S")),
    );
    params.insert(
        "endTime".into(),
        format!("{}", end_datetime.format("%Y-%m-%d %H:%M:%S")),
    );
    let path = format!("/site/{site_id}/power");
    let url = to_url(&path, &params);
    let reply_text = call_url(&url)?;

    trace!("Parsing json");
    let power: site::GeneratedPowerReply = serde_json::from_str(&reply_text)?;

    Ok(power.power)
}

#[test]
fn test_map_to_params() {
    let mut map = HashMap::new();
    map.insert("key".to_string(), "value".to_string());
    map.insert("key2".to_string(), "value2".to_string());

    let params = map_to_params(&map);
    // order of k/v-pairs not known
    assert!(params == "key=value&key2=value2" || params == "key2=value2&key=value");
}
