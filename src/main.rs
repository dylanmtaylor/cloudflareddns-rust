extern crate failure;
extern crate serde_json;
use failure::format_err;
use serde_json::json;
use std::env;
use std::net::IpAddr;
use std::str::FromStr;
use std::thread;
use std::time::Duration;

pub fn get_external_ip(api_endpoint: &str) -> Result<std::string::String, failure::Error> {
    let client = reqwest::blocking::Client::new();

    let res = client.get(api_endpoint).send()?;

    if res.status().is_success() {
        let body = res.text().unwrap();
        let ip = IpAddr::from_str(&body);

        if let Ok(_ip) = ip {
            // If parsing succeeded, return the IP address
            Ok(body)
        } else {
            // If parsing failed, return an error
            println!("Error: {} is not a valid IP address.", body);
            Err(format_err!("Error: {} is not a valid IP address.", body))
        }
    } else {
        Err(format_err!(
            "Error: Retrieving the IP address API endpoint failed: {}",
            res.error_for_status().unwrap_err()
        ))
    }
}

pub fn get_external_ipv6() -> Result<std::string::String, failure::Error> {
    // Allows users to optionally configure which endpoints are used, with a sensible default.
    let api_endpoint = env::var("CLOUDFLAREDDNS_IPV6_API_ENDPOINT")
        .unwrap_or("https://api6.ipify.org".to_string());
    get_external_ip(&api_endpoint)
}

pub fn get_external_ipv4() -> Result<std::string::String, failure::Error> {
    // Allows users to optionally configure which endpoints are used, with a sensible default.
    let api_endpoint =
        env::var("CLOUDFLAREDDNS_IPV4_API_ENDPOINT").unwrap_or("https://api.ipify.org".to_string());
    get_external_ip(&api_endpoint)
}

fn get_zone_id(user: &str, api_key: &str, zone_name: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let url = format!(
        "https://api.cloudflare.com/client/v4/zones?name={}",
        zone_name
    );
    println!("Url for GET request: {}", url);

    let res = client
        .get(&url)
        .header("X-Auth-Email", user)
        .header("X-Auth-Key", api_key)
        .header("Content-Type", "application/json")
        .send()?;

    if res.status().is_success() {
        let json = res.json::<serde_json::Value>().unwrap();
        let zones = json["result"].as_array().expect("Expected array of zones");
        let zone = &zones[0];
        let zone_id = zone["id"]
            .as_str()
            .expect("Expected zone ID to be a string");
        Ok(zone_id.to_string())
    } else {
        Err(res.error_for_status().unwrap_err())
    }
}

fn create_or_update_record(
    user: &str,
    api_key: &str,
    ip: &str,
    record_name: &str,
    record_type: &str,
    zone_id: &str,
) -> Result<(), reqwest::Error> {
    let client = reqwest::blocking::Client::new();

    // First we get all DNS records in the zone matching the name and type
    let url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records?name={}&type={}",
        zone_id, record_name, record_type
    );
    println!("Url for POST request: {}", url);

    let res = client
        .get(&url)
        .header("X-Auth-Email", user)
        .header("X-Auth-Key", api_key)
        .header("Content-Type", "application/json")
        .send()?;

    if res.status().is_success() {
        // Read the response body as a JSON value
        let json = res.json::<serde_json::Value>().unwrap();
        let records = json["result"].as_array().unwrap();
        if !records.is_empty() && records[0]["content"] == ip {
            println!(
                "The record is already correct. No need to do anything here!\n{}",
                records[0]
            );
            Ok(())
        } else if records.is_empty() {
            // We need to create the record
            let client = reqwest::blocking::Client::new();
            let post_url = format!(
                "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
                zone_id
            );
            let body = serde_json::json!({
                "type": record_type,
                "name": record_name,
                "content": ip,
                "ttl": 1,
                "proxied": false
            });

            println!("POST URL: {}\nPOST body: {}", post_url, body);

            let res = client
                .post(&post_url)
                .header("X-Auth-Email", user)
                .header("X-Auth-Key", api_key)
                .header("Content-Type", "application/json")
                .json(&body)
                .send()?;

            if res.status().is_success() {
                println!("Created a new record\n{}", res.text().unwrap());
                Ok(())
            } else {
                println!("Failed to create record.");
                Err(res.error_for_status().unwrap_err())
            }
        } else {
            // We need to put a new value in the record
            let put_url = format!(
                "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
                zone_id,
                records[0]["id"].as_str().unwrap()
            );

            // Send a PUT request to the API endpoint
            let body = json!({
                "type": record_type,
                "name": record_name,
                "content": ip,
                "ttl": 1,
                "proxied": false
            });

            println!("PUT URL: {}\nPUT body: {}", put_url, body);

            let res = client
                .put(&put_url)
                .header("X-Auth-Email", user)
                .header("X-Auth-Key", api_key)
                .header("Content-Type", "application/json")
                .json(&body)
                .send()?;

            println!("{}", res.text().unwrap());
            Ok(())
        }
    } else {
        Err(res.error_for_status().unwrap_err())
    }
}

fn check_ips_and_update_dns(
    user: &str,
    api_key: &str,
    hosts_vec: &Vec<&str>,
    zones_vec: &Vec<&str>,
    ipv4: bool,
    ipv6: bool,
) -> Result<(), failure::Error> {
    let external_ipv4 = if ipv4 {
        get_external_ipv4()?
    } else {
        "unused".to_owned()
    };
    println!("External IPv4 address: {}", external_ipv4);
    let external_ipv6 = if ipv6 {
        get_external_ipv6()?
    } else {
        "unused".to_owned()
    };
    println!("External IPv6 address: {}", external_ipv6);

    // Iterate over an enumerated value of a tuple of the matching host and zone
    for (host, zone) in hosts_vec.iter().zip(zones_vec.iter()) {
        // Call the get_zone_id function to get the zone ID for the current host.
        let zone_id = get_zone_id(user, api_key, zone)?;
        println!("Zone ID for zone {}: {}", zone, zone_id);

        if ipv4 {
            match create_or_update_record(user, api_key, &external_ipv4, host, "A", &zone_id) {
                Ok(_) => println!(
                    "Successfully updated A record for {} in zone {} in CloudFlare to {}",
                    host, zone, external_ipv4
                ),
                Err(e) => println!("Failed to create or update record: {}", e),
            }
        }

        if ipv6 {
            match create_or_update_record(user, api_key, &external_ipv6, host, "AAAA", &zone_id) {
                Ok(_) => println!(
                    "Successfully updated AAAA record for {} in zone {} in CloudFlare to {}",
                    host, zone, external_ipv6
                ),
                Err(e) => println!("Failed to create or update record: {}", e),
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), failure::Error> {
    let user = std::env::var("CLOUDFLAREDDNS_USER")
        .expect("CLOUDFLAREDDNS_USER environment variable not set");
    let api_key = std::env::var("CLOUDFLAREDDNS_APIKEY")
        .expect("CLOUDFLAREDDNS_APIKEY environment variable not set");
    let record_types = std::env::var("CLOUDFLAREDDNS_RECORDTYPES")
        .expect("CLOUDFLAREDDNS_RECORDTYPES environment variable not set");
    // Get repeat interval, with a default value of 0, which runs only once.
    let repeat_interval =
        std::env::var("CLOUDFLAREDDNS_REPEAT_INTERVAL").unwrap_or("0".to_string());
    // Parse this string value into a 64-bit unsigned integer.
    let repeat_interval: u64 = repeat_interval.parse().unwrap_or(0);
    let record_type_values = record_types.split(';').collect::<Vec<_>>();
    let ipv4 = record_type_values.contains(&"A");
    let ipv6 = record_type_values.contains(&"AAAA");
    // host and zones as parallel arrays with elements at the same index expected to go together
    let hosts = std::env::var("CLOUDFLAREDDNS_HOSTS")
        .expect("CLOUDFLAREDDNS_HOSTS environment variable not set");
    let zones = std::env::var("CLOUDFLAREDDNS_ZONES")
        .expect("CLOUDFLAREDDNS_ZONES environment variable not set");
    // Split the hosts and zones strings on the semicolon character into vectors.
    let hosts_vec = hosts.split(';').collect::<Vec<_>>();
    let zones_vec = zones.split(';').collect::<Vec<_>>();

    // If the lengths of hosts and zones not equal, return an error.
    if hosts_vec.len() != zones_vec.len() {
        return Err(format_err!(
            "Error: hosts and zones have different lengths. These need to match"
        ));
    } else if hosts_vec.is_empty() || zones_vec.is_empty() {
        return Err(format_err!(
            "Error: hosts and zones must both not be empty."
        ));
    }

    loop {
        check_ips_and_update_dns(&user, &api_key, &hosts_vec, &zones_vec, ipv4, ipv6)?;
        if repeat_interval <= 0 {
            break;
        }
        println!(
            "Done updating DNS. Sleeping for {} seconds ...",
            repeat_interval
        );
        thread::sleep(Duration::from_secs(repeat_interval));
    }

    Ok(())
}
