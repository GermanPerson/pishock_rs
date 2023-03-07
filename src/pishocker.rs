use crate::api_endpoints::PiShockOpCode;
use crate::errors::PiShockError;
use crate::{errors, PUBLIC_PISHOCK_API_BASE};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Represents a single [`PiShocker`] device. This struct should not be constructed directly, use [`PiShockAccount::get_shocker`] instead.
#[derive(Debug, Clone)]
pub struct PiShocker {
    pub(crate) share_code: String,
    pub(crate) http_client: reqwest::Client,
    pub(crate) api_key: String,
    pub(crate) api_username: String,
    pub(crate) app_name: String,
    pub(crate) api_server_url: String,
    pub(crate) metadata: Option<PiShockerMetadata>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PiShockerMetadata {
    pub client_id: i64,
    pub id: i64,
    pub name: String,
    pub paused: bool,
    pub max_intensity: i64,
    pub max_duration: i64,
    pub online: bool,
}

impl PiShocker {
    /// Creates a new `PiShocker` instance.
    /// This function should not be called directly, use [`PiShockAccount::get_shocker`] instead.
    #[must_use]
    pub fn new<S: Into<String>>(
        share_code: S,
        api_key: S,
        api_username: S,
        app_name: S,
    ) -> PiShocker {
        PiShocker {
            share_code: share_code.into(),
            http_client: reqwest::Client::new(),
            api_key: api_key.into(),
            api_username: api_username.into(),
            app_name: app_name.into(),
            api_server_url: PUBLIC_PISHOCK_API_BASE.to_string(),
            metadata: None,
        }
    }

    /// Sets the API server URL to use for requests
    /// This is intended for testing purposes only and not compiled in release builds
    #[cfg(test)]
    pub(crate) fn set_api_server_url<S: Into<String>>(&mut self, api_server_url: S) {
        self.api_server_url = api_server_url.into();
    }

    #[must_use]
    pub fn get_share_code(&self) -> String {
        self.share_code.clone()
    }

    /// Triggers a beep with the specified duration
    ///
    /// ```no_run
    /// # tokio_test::block_on(async {
    /// use std::time::Duration;
    /// use pishock_rs::PiShocker;
    /// use pishock_rs::PiShockAccount;
    ///
    /// let pishock_account = PiShockAccount::new("pishock_rs", "username", "apikey");
    ///
    /// let pishocker_instance = pishock_account.get_shocker("sharecode".to_string()).await.unwrap();
    ///
    /// // Beeps for 10 seconds
    /// pishocker_instance.beep(Duration::from_secs(10)).await.expect("Failed to beep");
    /// # });
    pub async fn beep(&self, duration: Duration) -> Result<(), PiShockError> {
        debug!("Beeping user for {} seconds", duration.as_secs());
        self.action_api_request(PiShockOpCode::Beep, 0, duration)
            .await?;

        Ok(())
    }

    /// Vibrates the shocker with the specified intensity and duration
    /// Intensity is a value between 1 and 100
    ///
    /// ```no_run
    /// # tokio_test::block_on(async {
    /// use std::time::Duration;
    /// use pishock_rs::PiShocker;
    /// use pishock_rs::PiShockAccount;
    ///
    /// let pishock_account = PiShockAccount::new("pishock_rs", "username", "apikey");
    ///
    /// let pishocker_instance = pishock_account.get_shocker("sharecode".to_string()).await.unwrap();
    ///
    /// // Shock the user with an intensity of 50 and a duration of 10 seconds
    /// pishocker_instance.vibrate(50, Duration::from_secs(10)).await.expect("Failed to vibrate");
    /// # });
    pub async fn vibrate(&self, intensity: u32, duration: Duration) -> Result<(), PiShockError> {
        info!(
            "Vibrating user with intensity {} and duration {} seconds",
            intensity,
            duration.as_secs()
        );
        self.action_api_request(PiShockOpCode::Vibrate, intensity, duration)
            .await?;

        Ok(())
    }

    /// Delivers a 300ms shock with the specified intensity
    /// Intensity is a value between 1 and the maximum intensity of the shocker (max 100)
    pub async fn mini_shock(&self, intensity: u32) -> Result<(), errors::PiShockError> {
        info!(
            "Mini shocking user with intensity {} and duration 300ms",
            intensity
        );
        self.shock(intensity, Duration::from_millis(300)).await?;

        Ok(())
    }

    /// <p style="background:rgba(255,181,77,0.16);padding:0.75em;">
    /// <strong>Warning:</strong> Shocks the user without any warning. Brat mode.
    /// </p>
    ///
    /// Refer to documentation of `shock_with_warning` for more information.
    pub async fn shock(
        &self,
        intensity: u32,
        duration: Duration,
    ) -> Result<(), errors::PiShockError> {
        info!(
            "Shocking user with intensity {} and duration {} seconds",
            intensity,
            duration.as_secs()
        );
        self.action_api_request(PiShockOpCode::Shock, intensity, duration)
            .await?;

        Ok(())
    }

    /// Shocks the user with a short soft warning vibration beforehand.
    /// This is the recommended way to shock someone.
    ///
    /// ```no_run
    /// # tokio_test::block_on(async {
    /// use std::time::Duration;
    /// use pishock_rs::PiShocker;
    /// use pishock_rs::PiShockAccount;
    ///
    /// let pishock_account = PiShockAccount::new("pishock_rs", "username", "apikey");
    /// let pishocker_instance = pishock_account.get_shocker("sharecode".to_string()).await.unwrap();
    ///
    /// // Shock the user with an intensity of 50 and a duration of 2 seconds
    /// pishocker_instance.shock_with_warning(50, Duration::from_secs(2)).await.expect("Failed to shock user");
    /// # })
    /// ```
    /// # Errors
    /// The maximum intensity may be below 100, depending on user settings. Make **sure** that you handle `PiShockError::InvalidIntensity` errors properly.
    ///
    /// The maximum duration is 15 seconds, but may be lower because of user settings. Make **sure** that you handle `PiShockError::InvalidDuration` errors properly.
    ///
    pub async fn shock_with_warning(
        &self,
        intensity: u32,
        duration: Duration,
    ) -> Result<(), PiShockError> {
        debug!("Sending warning vibration");
        self.vibrate(20, Duration::from_secs(1)).await?;

        tokio::time::sleep(Duration::from_millis(200)).await; // The firmware requires some delay between commands
        debug!("Sending shock");
        self.shock(intensity, duration).await?;

        Ok(())
    }

    /// Returns the name of the shocker
    #[must_use]
    pub fn get_shocker_name(&self) -> Option<String> {
        self.metadata.as_ref().map(|metadata| metadata.name.clone())
    }

    /// Returns the client ID
    #[must_use]
    pub fn get_client_id(&self) -> Option<i64> {
        self.metadata.as_ref().map(|metadata| metadata.client_id)
    }

    /// Returns the shocker ID
    #[must_use]
    pub fn get_shocker_id(&self) -> Option<i64> {
        self.metadata.as_ref().map(|metadata| metadata.id)
    }

    /// Returns the maximum shock intensity
    #[must_use]
    pub fn get_max_intensity(&self) -> Option<i64> {
        self.metadata
            .as_ref()
            .map(|metadata| metadata.max_intensity)
    }

    /// Returns the maximum shock duration
    #[must_use]
    pub fn get_max_duration(&self) -> Option<Duration> {
        self.metadata
            .as_ref()
            .map(|metadata| Duration::from_secs(metadata.max_duration as u64))
    }

    /// Returns whether the shocker is online or not
    #[must_use]
    pub fn get_shocker_online(&self) -> Option<bool> {
        self.metadata.as_ref().map(|metadata| metadata.online)
    }

    /// Returns whether the shocker is paused or not
    #[must_use]
    pub fn get_shocker_paused(&self) -> Option<bool> {
        self.metadata.as_ref().map(|metadata| metadata.paused)
    }

    // Metadata-supported intensity and duration checks that are used by the internal API request function
    pub(crate) fn max_intensity_error_triggered(&self, intensity: u32) -> Option<PiShockError> {
        if self.get_max_intensity().is_some()
            && intensity > self.get_max_intensity().unwrap() as u32
        {
            Some(PiShockError::InvalidIntensity(
                self.get_max_intensity().unwrap() as u32,
            ))
        } else {
            None
        }
    }

    pub(crate) fn max_duration_error_triggered(&self, duration: Duration) -> Option<PiShockError> {
        if self.get_max_duration().is_some() && duration > self.get_max_duration().unwrap() {
            Some(PiShockError::InvalidDuration(
                self.get_max_duration().unwrap().as_secs() as u32,
            ))
        } else {
            None
        }
    }
}
