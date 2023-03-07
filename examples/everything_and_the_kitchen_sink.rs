extern crate pishock_rs;

use log::{error, info, log};
use pishock_rs::errors::PiShockError;
use pishock_rs::PiShocker;
use simplelog::{Config, LevelFilter, TerminalMode};
use std::io::Write;
use std::process::exit;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    simplelog::TermLogger::init(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .unwrap();

    println!("Simple example of using the PiShock API - env variable control");

    let shock_intensity = std::env::var("PISHOCK_INTENSITY").unwrap_or("20".to_string());
    let shock_duration = std::env::var("PISHOCK_DURATION").unwrap_or("1".to_string());
    let shocker_share_code = std::env::var("PISHOCK_SHARECODE").unwrap_or(String::new());
    let shocker_api_key = std::env::var("PISHOCK_APIKEY").unwrap_or(String::new());
    let shocker_api_username = std::env::var("PISHOCK_USERNAME").unwrap_or(String::new());

    println!("Shock intensity (PISHOCK_INTENSITY): {shock_intensity}");
    println!("Shock duration (PISHOCK_DURATION): {shock_duration}");
    println!("Shocker share code (PISHOCK_SHARECODE): {shocker_share_code}");
    println!("Shocker API key (PISHOCK_APIKEY): {shocker_api_key}");
    println!("Shocker API username (PISHOCK_USERNAME): {shocker_api_username}");

    if shocker_share_code.is_empty()
        || shocker_api_key.is_empty()
        || shocker_api_username.is_empty()
    {
        error!("PISHOCK_SHARECODE, PISHOCK_APIKEY and PISHOCK_USERNAME must be set");
        exit(1);
    }

    // Create a new PiShockAccount instance
    let pishock_account = pishock_rs::PiShockAccount::new(
        "pishock_rs example".to_string(),
        shocker_api_username,
        shocker_api_key,
    );

    // Get a PiShocker instance
    let pishocker_instance: PiShocker = match pishock_account.get_shocker(shocker_share_code).await
    {
        Ok(pishock_instance) => pishock_instance,
        Err(e) => {
            error!("Failed to get PiShocker instance: {e}");
            exit(1);
        }
    };

    // Print all the PiShocker's details
    println!("PiShocker details:");
    println!("  Name: {}", pishocker_instance.get_shocker_name().unwrap());
    println!(
        "  Max intensity: {}",
        pishocker_instance.get_max_intensity().unwrap()
    );
    println!(
        "  Max duration: {:#?}",
        pishocker_instance.get_max_duration().unwrap()
    );
    println!(
        "  Client ID: {}",
        pishocker_instance.get_client_id().unwrap()
    );
    println!(
        "  Online: {}",
        pishocker_instance.get_shocker_online().unwrap()
    );
    println!(
        "  Paused: {}",
        pishocker_instance.get_shocker_paused().unwrap()
    );
}
