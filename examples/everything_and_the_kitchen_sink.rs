extern crate pishock_rs;

use log::{error, info, log};
use pishock_rs::interpolation::ShockPoint;
use pishock_rs::PiShocker;
use simplelog::{Config, LevelFilter, TerminalMode};
use std::process::exit;
use std::time::Duration;

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

    let shocker_share_code = std::env::var("PISHOCK_SHARECODE").unwrap_or(String::new());
    let shocker_api_key = std::env::var("PISHOCK_APIKEY").unwrap_or(String::new());
    let shocker_api_username = std::env::var("PISHOCK_USERNAME").unwrap_or(String::new());

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

    let test_pishocker_instance = pishock_account
        .get_shocker_without_verification(shocker_share_code.clone())
        .await
        .unwrap();
    test_pishocker_instance
        .shock_curve(vec![
            ShockPoint::new(Duration::from_secs(2), 100),
            ShockPoint::new(Duration::from_secs(3), 30),
            ShockPoint::new(Duration::from_secs(1), 1),
            ShockPoint::new(Duration::from_secs(3), 90),
            ShockPoint::new(Duration::from_secs(4), 1),
        ])
        .await
        .unwrap();

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
