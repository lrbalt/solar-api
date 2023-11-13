use chrono::{Duration, Local};
use solar_api::{data_period, details, energy, list, overview, power, DataPeriod, TimeUnit};
use std::{env, error::Error};
use uom::{
    fmt::DisplayStyle,
    si::{
        energy::{megawatt_hour, watt_hour},
        power::{kilowatt, watt},
    },
};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("usage: use_api <API_KEY> <SITE_ID>");
        return Ok(());
    }
    let api_key: &str = args[1].as_ref();
    let site_id: u32 = args[2].parse()?;
    println!("Accessing API using {api_key} for site {site_id}");

    println!("Getting information of all sites of customer");
    for site in list(api_key)? {
        println!("Id: {}\tName: {}", site.id, site.name);
    }

    println!("Getting information of site {site_id}");
    let site_details = details(api_key, site_id)?;
    println!(
        "Id = {}\tstatus: {}\t peak_power: {}",
        site_details.id,
        site_details.status,
        site_details
            .peak_power
            .into_format_args(kilowatt, uom::fmt::DisplayStyle::Description)
    );

    println!("Getting period of available data of site {site_id}");
    let data_period = data_period(api_key, site_id)?;
    println!(
        "Data available from {} until {}",
        data_period.start_date, data_period.end_date
    );

    println!("Getting overview of site {site_id}");
    let overview = overview(api_key, site_id)?;
    println!(
        "Site generated {:.2} since installation and is currently generating {:.2}",
        overview
            .life_time_data
            .energy
            .into_format_args(megawatt_hour, DisplayStyle::Abbreviation),
        overview
            .current_power
            .power
            .into_format_args(watt, DisplayStyle::Description)
    );

    println!("Getting energy generation of past day");
    let now = Local::now().naive_local();
    let period: DataPeriod = DataPeriod {
        start_date: now.date(),
        end_date: now.date(),
    };
    let time_unit = TimeUnit::Hour;
    let energy = energy(api_key, site_id, period, time_unit)?;
    for e in energy.values() {
        println!(
            "\t{} - {}",
            e.date,
            e.value
                .map(|v| format!(
                    "{}",
                    v.into_format_args(watt_hour, DisplayStyle::Abbreviation)
                ))
                .unwrap_or_else(|| "No value".to_string())
        );
    }
    println!("Getting power generation of past hour");
    let now = Local::now().naive_local();
    let power = power(api_key, site_id, now - Duration::hours(1), now)?;
    for e in power.values() {
        println!(
            "\t{} - {}",
            e.date,
            e.value
                .map(|v| format!("{}", v.into_format_args(watt, DisplayStyle::Description)))
                .unwrap_or_else(|| "No value".to_string())
        );
    }

    Ok(())
}
