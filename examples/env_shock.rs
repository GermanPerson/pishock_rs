extern crate pishock_rs;

use std::process::exit;
use log::error;
use simplelog::{Config, LevelFilter, TermLogger, TerminalMode};

#[tokio::main]
async fn main() {
    simplelog::TermLogger::init(LevelFilter::Debug, Config::default(), TerminalMode::Mixed, simplelog::ColorChoice::Auto).unwrap();

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

    if shocker_share_code.is_empty() || shocker_api_key.is_empty() || shocker_api_username.is_empty() {
        error!("PISHOCK_SHARECODE, PISHOCK_APIKEY and PISHOCK_USERNAME must be set");
        exit(1);
    }

    // Create a new PiShockController instance
    let pishock_controller = pishock_rs::PiShockController::new("pishock_rs example".to_string(), shocker_api_username, shocker_api_key);

    // Get a PiShocker instance
    let pishocker_instance = match pishock_controller.get_shocker(shocker_share_code, true).await {
        Ok(pishock_instance) => pishock_instance,
        Err(e) => {
            error!("Failed to get PiShocker instance: {e}");
            exit(1);
        }
    };

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // Shock the user with given intensity and duration
    match pishocker_instance.shock_with_warning(
        shock_intensity.parse::<u32>().unwrap(),
        std::time::Duration::from_secs(shock_duration.parse::<u64>().unwrap()))
        .await {
        Ok(_) => println!("Shock successful"),
        Err(e) => error!("Shock failed: {e}"),
    }


}