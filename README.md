# Solar API
Rust library for accessing the Solar Edge API. This library uses the API documentation found [here](https://knowledge-center.solaredge.com/en/search?search=api&sort_by=search_api_relevance)

# API Key and Site ID
To access the data of your installation, you need to get an API key. You can get this from the SolardEdge Monitoring Portal. Log in with your SolarEdge Account, go to the Admin section, Site Access tab and activate API access. Mark the checkbox and you will see the API Key and Site ID

# Rate limited
Please be aware that the API is rate limited, i.e. it will block requests after reaching a maximum of requests in an hour. It will be available again after that hour. Also note that the measurements seem to be limited to one per fifteen minutes. You can consider scheduling a read of data ±15 minutes after the timestamp of last read measurement. For example you can use a duration of 15m 10s:

```rust
let next_update = last_updated_datetime + Duration::seconds(15 * 60 + 10);
```

There is a convenience method to help with this:

```rust
let site_overview: Overview = overview(api_key, site_id);
let (next_update, duration_from_now) = site_overview.estimated_next_update();
```

Please note that sometimes the API is a bit later. The `duration_from_now` can be negative then and you have to wait a bit more like in the example below. Please note that `checked_add` is needed here to handle adding negative `duration_from_now`.
 
```rust
let site_overview: Overview = overview(api_key, site_id);
let (next_update, duration_from_now) = site_overview.estimated_next_update();

let next = Instant::now()
    .checked_add(Duration::from_secs(duration_from_now as u64))
    .unwrap_or(Instant::now() + Duration::from_secs(30));

// wait next or set timeout at next_update before 
// getting power or energy data
```

# Using the example in this crate
The example will call several API methods. To run it, use

```rust
cargo run --example use_api -- <API_KEY> <SITE_ID>
```

To see the http request and response use

```rust
RUST_LOG=solar_api=trace cargo run --example use_api -- <API_KEY> <SITE_ID>
```

# Status
* Site Data API
    * [x] Site List
    * [x] Site Details
    * [x] Site Data: Start and End Dates
    * [ ] Site Data: Bulk Version
    * [x] Site Energy
    * [ ] Site Energy: Bulk Version 
    * [x] Site Energy – Time Period
    * [ ] Site Energy – Time Period: Bulk Version
    * [x] Site Power
    * [ ]Site Power: Bulk version
    * [x] Site Overview
    * [ ] Site Overview: Bulk Version
    * [ ] Site Power - Detailed
    * [ ] Site Energy - Detailed
    * [ ] Site Power Flow
    * [ ] Storage Information
    * [ ] Site Image
    * [ ] Site Environmental Benefits
    * [ ] Installer Logo Image
* Site Equipment API
    * [ ] Components List
    * [ ] Inventory
    * [ ] Inverter Technical Data
    * [ ] Equipment Change Log
* [ ] Account List API
* Meters API
    * [ ] Get Meters Data
* Sensors API
    * [ ] Get Sensor List
    * [ ] Get Sensor Data
* Data Types
    * [x] Time Unit
    * [ ] Site Status
    * [ ] Site Type