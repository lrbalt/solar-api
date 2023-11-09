use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use uom::si::{
    energy::watt_hour,
    f64::{Energy, Power},
    power::{kilowatt, watt},
};

pub const REFRESH_TIME_IN_M: i64 = 15;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SitesReply {
    sites: Sites,
}

impl SitesReply {
    pub fn sites(&self) -> &Vec<Site> {
        &self.sites.site
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Sites {
    #[serde(rename = "count")]
    _count: u32,
    site: Vec<Site>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SiteDetails {
    pub details: Site,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Site {
    /// the site id
    pub id: u32,
    /// the site name
    pub name: String,
    /// the account this site belongs to
    #[serde(rename = "accountId")]
    pub account_id: u32,
    /// the site status
    pub status: String,
    /// site peak power
    #[serde(rename = "peakPower", deserialize_with = "parse_power_kw")]
    pub peak_power: Power,
    #[serde(rename = "lastUpdateTime", deserialize_with = "parse_date")]
    pub last_update_time: chrono::NaiveDate,
    /// site installation date
    #[serde(rename = "installationDate", deserialize_with = "parse_date")]
    pub installation_date: chrono::NaiveDate, 
    /// permission to operate date
    #[serde(rename = "ptoDate")]
    pub pto_date: Option<String>,
    pub notes: String,
    /// site type
    #[serde(rename = "type")]
    pub site_type: String,
    /// includes country, state, city, address, secondary address, time zone and zip
    pub location: Location,
    #[serde(rename = "primaryModule")]
    pub primary_module: PrimaryModule,
    pub uris: HashMap<String, String>,
    ///  includes if this site is public and its public name
    #[serde(rename = "publicSettings")]
    pub public_settings: PublicSettings,
}

/// Location of a site
#[derive(Debug, Clone, Deserialize)]
pub struct Location {
    pub country: String,
    pub city: String,
    pub address: String,
    pub zip: String,
    #[serde(rename = "timeZone")]
    pub time_zone: String,
    #[serde(rename = "countryCode")]
    pub country_code: String,
}

/// The information about the model of the primary module of the site
#[derive(Debug, Clone, Deserialize)]
pub struct PrimaryModule {
    #[serde(rename = "manufacturerName")]
    pub manufacturer_name: String,
    #[serde(rename = "modelName")]
    pub model_name: String,
    #[serde(rename = "maximumPower", deserialize_with = "parse_power_kw")]
    pub maximum_power: Power,
    #[serde(rename = "temperatureCoef")]
    pub temperature_coef: f32,
}

/// Setting showing if information about this site is public
#[derive(Debug, Clone, Deserialize)]
pub struct PublicSettings {
    #[serde(rename = "isPublic")]
    pub public: bool,
}

/// The period defined by start_date and end_date that this site is producting energy
#[derive(Debug, Clone, Deserialize)]
pub struct DataPeriod {
    #[serde(rename = "startDate", deserialize_with = "parse_date")]
    pub start_date: chrono::NaiveDate,
    #[serde(rename = "endDate", deserialize_with = "parse_date")]
    pub end_date: chrono::NaiveDate,
}

impl DataPeriod {
    /// create a formatted [`String`] for the start date 
    /// in `%Y-%m-%d` format, i.e. `2023-11-9` for november 9th 2023
    pub fn formatted_start_date(&self) -> String {
        Self::formatted_date(&self.start_date)
    }

    /// create a formatted [`String`] for the end date 
    /// in `%Y-%m-%d` format, i.e. `2023-11-9` for november 9th 2023
    pub fn formatted_end_date(&self) -> String {
        Self::formatted_date(&self.end_date)
    }

    fn formatted_date(date: &chrono::NaiveDate) -> String {
        date.format("%Y-%m-%d").to_string()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct DataPeriodReply {
    #[serde(rename = "dataPeriod")]
    pub(crate) data_period: DataPeriod,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct OverviewReply {
    pub(crate) overview: Overview,
}

/// The overview of a site includes the site current power, daily energy, monthly energy, yearly energy and life time energy.
#[derive(Debug, Clone, Deserialize)]
pub struct Overview {
    #[serde(rename = "lastUpdateTime", deserialize_with = "parse_date_time")]
    pub last_updated_time: chrono::NaiveDateTime,
    #[serde(rename = "lifeTimeData")]
    pub life_time_data: TimeData,
    #[serde(rename = "lastYearData")]
    pub last_year_data: TimeData,
    #[serde(rename = "lastMonthData")]
    pub last_month_data: TimeData,
    #[serde(rename = "lastDayData")]
    pub last_day_data: TimeData,
    #[serde(rename = "currentPower")]
    pub current_power: GeneratedPower,
    #[serde(rename = "measuredBy")]
    pub measured_by: String,
}

impl Overview {
    /// Calculates the next timestamp and the duration from now when new data 
    /// should be available on the API. It uses `last_update_time` and 15 
    /// minutes and 10 seconds as delta between updates
    pub fn estimated_next_update(&self) -> (chrono::NaiveDateTime, chrono::Duration) {
        // add 10s extra time
        let next = self.last_updated_time + chrono::Duration::seconds(REFRESH_TIME_IN_M * 60 + 10);
        let delta = next - chrono::Local::now().naive_local();
        (next, delta)
    }
}

/// Amount of [`Energy`] and optional the revenue of this energy
#[derive(Debug, Clone, Deserialize)]
pub struct TimeData {
    #[serde(deserialize_with = "parse_energy_wh")]
    pub energy: Energy,
    pub revenue: Option<f32>,
}

/// Generated power
#[derive(Debug, Clone, Deserialize)]
pub struct GeneratedPower {
    #[serde(deserialize_with = "parse_power_kw")]
    pub power: Power,
}

#[derive(Debug, Clone, Deserialize)]
pub enum TimeUnit {
    QuarterOfAnHour,
    Hour,
    Day,
    Week,
    Month,
    Year,
}

const QUARTER_OF_AN_HOUR: &str = "QUARTER_OF_AN_HOUR";
const HOUR: &str = "HOUR";
const DAY: &str = "DAY";
const WEEK: &str = "WEEK";
const MONTH: &str = "MONTH";
const YEAR: &str = "YEAR";

impl TimeUnit {
    pub fn to_param(&self) -> &'static str {
        match self {
            TimeUnit::QuarterOfAnHour => QUARTER_OF_AN_HOUR,
            TimeUnit::Hour => HOUR,
            TimeUnit::Day => DAY,
            TimeUnit::Week => WEEK,
            TimeUnit::Month => MONTH,
            TimeUnit::Year => YEAR,
        }
    }

    pub fn from_const<'de, D>(deserializer: D) -> Result<TimeUnit, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        match s.as_str() {
            QUARTER_OF_AN_HOUR => Ok(TimeUnit::QuarterOfAnHour),
            HOUR => Ok(TimeUnit::Hour),
            DAY => Ok(TimeUnit::Day),
            WEEK => Ok(TimeUnit::Week),
            MONTH => Ok(TimeUnit::Month),
            YEAR => Ok(TimeUnit::Year),
            _ => Err(serde::de::Error::custom("Cannot parse value")),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct GeneratedEnergyReply {
    pub(crate) energy: GeneratedEnergy,
}

/// Contains all values of the generated energy per time unit
#[derive(Debug, Clone, Deserialize)]
pub struct GeneratedEnergy {
    #[serde(rename = "timeUnit", deserialize_with = "TimeUnit::from_const")]
    pub time_unit: TimeUnit,
    unit: String,
    values: Vec<RawGeneratedEnergyValue>,
}

impl GeneratedEnergy {
    /// returns the timestamped energy values
    pub fn values(&self) -> Vec<GeneratedEnergyValue> {
        self.values
            .iter()
            .map(|raw| raw.convert(&self.unit))
            .collect()
    }
}

// struct used to parse reply from API. Can be converted to 
//[`GeneratedEnergyValue`] to contain correct unit of measurement 
// using the unit value returned by [`GeneratedEnergy`]
#[derive(Debug, Clone, Deserialize, Copy)]
struct RawGeneratedEnergyValue {
    #[serde(deserialize_with = "parse_date_time")]
    date: chrono::NaiveDateTime,
    value: Option<f64>,
}

impl RawGeneratedEnergyValue {
    // converts f64 value to [`Energy`] using supplied `unit`. 
    // Currenty only `Wh` is supported
    fn convert(&self, unit: &str) -> GeneratedEnergyValue {
        let value = match unit {
            "Wh" => self.value.map(Energy::new::<watt_hour>),
            _ => todo!("unsupported unit: {unit}"),
        };
        GeneratedEnergyValue {
            date: self.date,
            value,
        }
    }
}

/// A timestamped [`Energy`] value. The value may be None when there wasn't a 
/// value at that timestamp
#[derive(Debug, Clone, Copy)]
pub struct GeneratedEnergyValue {
    /// timestamp of value
    pub date: chrono::NaiveDateTime,
    /// the value measures at the timestamp or None if there wasn't a value at
    /// that timestamp
    pub value: Option<Energy>,
}

// struct used to parse the API reply for Power
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct GeneratedPowerReply {
    pub(crate) power: GeneratedPowerPerTimeUnit,
}

/// Contains all values of the generated power per time unit
#[derive(Debug, Clone, Deserialize)]
pub struct GeneratedPowerPerTimeUnit {
    #[serde(rename = "timeUnit", deserialize_with = "TimeUnit::from_const")]
    pub time_unit: TimeUnit,
    unit: String,
    values: Vec<RawGeneratedPowerValue>,
}

impl GeneratedPowerPerTimeUnit {
    /// returns all Power values that were present in the time period
    pub fn values(&self) -> Vec<GeneratedPowerValue> {
        self.values
            .iter()
            .map(|raw| raw.convert(&self.unit))
            .collect()
    }
}

#[derive(Debug, Clone, Deserialize)]
struct RawGeneratedPowerValue {
    #[serde(deserialize_with = "parse_date_time")]
    date: chrono::NaiveDateTime,
    value: Option<f64>,
}

impl RawGeneratedPowerValue {
    // converts f64 value to [`Power`] using supplied `unit`. 
    // Currenty only `W` is supported
    pub fn convert(&self, unit: &str) -> GeneratedPowerValue {
        let value: Option<Power> = match unit {
            "W" => self.value.map(Power::new::<watt>),
            _ => todo!("unsupported unit: {unit}"),
        };
        GeneratedPowerValue {
            date: self.date,
            value,
        }
    }
}

/// A timestamped [`Power`] value. The value may be None when there wasn't a 
/// value at that timestamp
#[derive(Debug, Clone)]
pub struct GeneratedPowerValue {
    pub date: chrono::NaiveDateTime,
    pub value: Option<Power>,
}

// parse a datetime value that the API returned to a [`NaiveDateTime`]
fn parse_date_time<'de, D>(deserializer: D) -> Result<chrono::NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
        .map_err(|_| serde::de::Error::custom("Cannot parse value"))
}

// parse a datetime value that the API returned to a [`NaiveDate`]
fn parse_date<'de, D>(deserializer: D) -> Result<chrono::NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")
        .map_err(|_| serde::de::Error::custom("Cannot parse value"))
}

// parse a float value that the API returned to a [`Power`] value. Assumes the value is in kilowatt
fn parse_power_kw<'de, D>(deserializer: D) -> Result<Power, D::Error>
where
    D: Deserializer<'de>,
{
    let value: f64 = f64::deserialize(deserializer)?;
    Ok(Power::new::<kilowatt>(value))
}

// parse a float value that the API returned to a [`Energy`] value. Assumes the value is in watt-hours
fn parse_energy_wh<'de, D>(deserializer: D) -> Result<Energy, D::Error>
where
    D: Deserializer<'de>,
{
    let value: f64 = f64::deserialize(deserializer)?;
    Ok(Energy::new::<watt_hour>(value))
}

#[test]
fn test_parse_sites_data() {
    let output = r#"
       {"sites":{
           "count":1,
           "site":[
               {"id":1234123,
                "name":"MySiteName",
                "accountId":123456,
                "status":"Active",
                "peakPower":7.41,
                "lastUpdateTime":"2021-04-29",
                "installationDate":"2021-02-25",
                "ptoDate":null,
                "notes":"",
                "type":"Optimizers & Inverters",
                "location":{
                    "country":"Netherlands",
                    "city":"A city",
                    "address":"Some address",
                    "zip":"zipy",
                    "timeZone":"Europe/Amsterdam",
                    "countryCode":"NL"
                },
                "primaryModule":{
                    "manufacturerName":"JinkoSolar",
                    "modelName":"390",
                    "maximumPower":0.0,
                    "temperatureCoef":0.0
                },
                "uris":{
                    "SITE_IMAGE":"/site/1234123/siteImage/file12341234.jpg",
                    "DATA_PERIOD":"/site/1234123/dataPeriod",
                    "DETAILS":"/site/1234123/details",
                    "OVERVIEW":"/site/1234123/overview"
                },
                "publicSettings":{
                    "isPublic":false
                }}
            ]
        }
    }"#;

    let reply: SitesReply = serde_json::from_str(output).unwrap();
    println!("{:?}", reply);
    assert_eq!(reply.sites._count, 1);
    let power = Power::new::<kilowatt>(7.41);
    assert_eq!(power, reply.sites.site[0].peak_power);
}

#[test]
fn test_parse_data_period() {
    let reply = r#"{"dataPeriod":{"startDate":"2021-02-25","endDate":"2021-05-03"}}"#;
    println!("{}", reply);
    let parsed: DataPeriodReply = serde_json::from_str(reply).unwrap();
    assert_eq!("2021-02-25", parsed.data_period.formatted_start_date());
    assert_eq!("2021-05-03", parsed.data_period.formatted_end_date());
}

#[test]
fn test_energy() {
    use uom::si::energy::watt_hour;

    let reply = r#"
    {"energy":{
        "timeUnit":"MONTH",
        "unit":"Wh",
        "measuredBy":"INVERTER",
        "values":[
            {"date":"2021-02-01 00:00:00","value":45718.0},
            {"date":"2021-03-01 00:00:00","value":504857.0},
            {"date":"2021-04-01 00:00:00","value":800476.0},
            {"date":"2021-05-01 00:00:00","value":89913.0}]}}
    "#;

    let parsed: GeneratedEnergyReply = serde_json::from_str(reply).unwrap();
    assert_eq!(
        45718.0,
        parsed.energy.values()[0]
            .value
            .map(|e| e.get::<watt_hour>())
            .unwrap()
    );
}

#[test]
fn test_overview() {
    let reply = r#"
    {"overview":{
        "lastUpdateTime":"2023-11-09 10:28:56",
        "lifeTimeData":{"energy":1.9191678E7},
        "lastYearData":{"energy":6143745.0},
        "lastMonthData":{"energy":38709.0},
        "lastDayData":{"energy":2028.0},
        "currentPower":{"power":1173.7279},
        "measuredBy":"INVERTER"}
    }
    "#;

    let parsed: OverviewReply = serde_json::from_str(reply).unwrap();
    assert_eq!(
        Energy::new::<watt_hour>(1.9191678E7),
        parsed.overview.life_time_data.energy
    );
    assert_eq!(
        Power::new::<kilowatt>(1173.7279),
        parsed.overview.current_power.power
    );
}

#[test]
fn test_energy_in_period() {
    let reply = r#"
    {"energy":{
        "timeUnit":"HOUR",
        "unit":"Wh",
        "measuredBy":"INVERTER",
        "values":[
            {"date":"2023-11-09 00:00:00","value":null},
            {"date":"2023-11-09 01:00:00","value":null},
            {"date":"2023-11-09 02:00:00","value":null},
            {"date":"2023-11-09 03:00:00","value":null},
            {"date":"2023-11-09 04:00:00","value":0.0},
            {"date":"2023-11-09 05:00:00","value":null},
            {"date":"2023-11-09 06:00:00","value":null},
            {"date":"2023-11-09 07:00:00","value":0.0},
            {"date":"2023-11-09 08:00:00","value":256.0},
            {"date":"2023-11-09 09:00:00","value":827.0},
            {"date":"2023-11-09 10:00:00","value":1390.0},
            {"date":"2023-11-09 11:00:00","value":222.0},
            {"date":"2023-11-09 12:00:00","value":null},
            {"date":"2023-11-09 13:00:00","value":null},
            {"date":"2023-11-09 14:00:00","value":null},
            {"date":"2023-11-09 15:00:00","value":null},
            {"date":"2023-11-09 16:00:00","value":null},
            {"date":"2023-11-09 17:00:00","value":null},
            {"date":"2023-11-09 18:00:00","value":null},
            {"date":"2023-11-09 19:00:00","value":null},
            {"date":"2023-11-09 20:00:00","value":null},
            {"date":"2023-11-09 21:00:00","value":null},
            {"date":"2023-11-09 22:00:00","value":null},
            {"date":"2023-11-09 23:00:00","value":null}
            ]
        }
    }
    "#;

    let parsed: GeneratedEnergyReply = serde_json::from_str(reply).unwrap();
    assert_eq!(24, parsed.energy.values().len());
    assert_eq!(
        Some(Energy::new::<watt_hour>(222.0)),
        parsed.energy.values()[11].value
    );
}

#[test]
fn test_power_in_period() {
    let reply = r#"
    {"power":{
        "timeUnit":"QUARTER_OF_AN_HOUR",
        "unit":"W",
        "measuredBy":"INVERTER",
        "values":[
            {"date":"2023-11-09 12:15:00","value":761.538},
            {"date":"2023-11-09 12:30:00","value":822.26117},
            {"date":"2023-11-09 12:45:00","value":746.9589},
            {"date":"2023-11-09 13:00:00","value":563.11},
            {"date":"2023-11-09 13:15:00","value":554.06836}
        ]
    }}
    "#;

    let parsed: GeneratedPowerReply = serde_json::from_str(reply).unwrap();
    assert_eq!(5, parsed.power.values().len());
    assert_eq!(
        Some(Power::new::<watt>(761.538)),
        parsed.power.values()[0].value
    );
}
