pub mod errors;

use crate::errors::error_to_pishock_error;
use log::{debug, info};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// The main controller for the PiShock API.
///
/// Construct a new instance with [`PiShockController::new`].
pub struct PiShockController {
    app_name: String,
    api_username: String,
    api_key: String,
}

impl PiShockController {
    #[must_use]
    pub fn new<S: Into<String>>(api_name: S, api_username: S, api_key: S) -> PiShockController {
        PiShockController {
            app_name: api_name.into(),
            api_username: api_username.into(),
            api_key: api_key.into(),
        }
    }

    /// Returns a `PiShocker` instance for the specified share code
    ///
    /// Warning: This function will vibrate the shocker softly for one second if `verify_connection` is true
    /// ```
    /// # use pishock_rs::PiShockController;
    ///  let pishock_controller = PiShockController::new("pishock_rs", "username", "apikey");
    /// ```
    /// # Errors
    /// Will only return an error if `verify_connection` is true and an error occurs while trying to vibrate the shocker
    pub async fn get_shocker<S: Into<String>>(
        &self,
        share_code: S,
        verify_connection: bool,
    ) -> Result<PiShocker, errors::PiShockError> {
        let pishock_instance = PiShocker::new(
            share_code.into(),
            self.api_key.clone(),
            self.api_username.clone(),
            self.app_name.clone(),
        );

        if verify_connection {
            pishock_instance.vibrate(20, Duration::from_secs(1)).await?;
            info!("Successfully connected to shocker, connection verified");
        }

        Ok(pishock_instance)
    }
}

#[derive(Clone, Copy)]
enum PiShockOpCode {
    Shock = 0,
    Vibrate = 1,
    Beep = 2,
}

static PUBLIC_PISHOCK_API_URL: &str = "https://do.pishock.com/api/apioperate/";

/// Represents a single PiShocker device. This struct should not be constructed directly, use [`PiShockController::get_shocker`] instead.
#[derive(Debug, Clone)]
pub struct PiShocker {
    share_code: String,
    http_client: reqwest::Client,
    api_key: String,
    api_username: String,
    app_name: String,
    api_server_url: String,
}

impl PiShocker {
    /// Creates a new `PiShocker` instance.
    /// This function should not be called directly, use [`PiShockController::get_shocker`] instead.
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
            api_server_url: PUBLIC_PISHOCK_API_URL.to_string(),
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
    /// use pishock_rs::PiShockController;
    ///
    /// let pishock_controller = PiShockController::new("pishock_rs", "username", "apikey");
    ///
    /// let pishocker_instance = pishock_controller.get_shocker("sharecode".to_string(), true).await.unwrap();
    ///
    /// // Beeps for 10 seconds
    /// pishocker_instance.beep(Duration::from_secs(10)).await.expect("Failed to beep");
    /// # });
    pub async fn beep(&self, duration: Duration) -> Result<(), errors::PiShockError> {
        debug!("Beeping user for {} seconds", duration.as_secs());
        self.api_request(PiShockOpCode::Beep, 0, duration.as_secs() as u32)
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
    /// use pishock_rs::PiShockController;
    ///
    /// let pishock_controller = PiShockController::new("pishock_rs", "username", "apikey");
    ///
    /// let pishocker_instance = pishock_controller.get_shocker("sharecode".to_string(), true).await.unwrap();
    ///
    /// // Shock the user with an intensity of 50 and a duration of 10 seconds
    /// pishocker_instance.vibrate(50, Duration::from_secs(10)).await.expect("Failed to vibrate");
    /// # });
    pub async fn vibrate(
        &self,
        intensity: u32,
        duration: Duration,
    ) -> Result<(), errors::PiShockError> {
        if !(1..=100).contains(&intensity) {
            return Err(errors::PiShockError::InvalidIntensity(100));
        }

        info!(
            "Vibrating user with intensity {} and duration {} seconds",
            intensity,
            duration.as_secs()
        );
        self.api_request(PiShockOpCode::Vibrate, intensity, duration.as_secs() as u32)
            .await?;

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
        if duration.as_secs() < 1 || duration.as_secs() > 15 {
            return Err(errors::PiShockError::InvalidDuration(
                duration.as_secs() as u32
            ));
        }

        if !(1..=100).contains(&intensity) {
            return Err(errors::PiShockError::InvalidIntensity(100));
        }

        info!(
            "Shocking user with intensity {} and duration {} seconds",
            intensity,
            duration.as_secs()
        );
        self.api_request(PiShockOpCode::Shock, intensity, duration.as_secs() as u32)
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
    /// use pishock_rs::PiShockController;
    ///
    /// let pishock_controller = PiShockController::new("pishock_rs", "username", "apikey");
    /// let pishocker_instance = pishock_controller.get_shocker("sharecode".to_string(), true).await.unwrap();
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
    ) -> Result<(), errors::PiShockError> {
        debug!("Sending warning vibration");
        self.vibrate(20, Duration::from_secs(1)).await?;
        tokio::time::sleep(Duration::from_millis(200)).await; // The firmware requires some delay between commands
        debug!("Sending shock");
        self.shock(intensity, duration).await?;

        Ok(())
    }

    async fn api_request(
        &self,
        op_code: PiShockOpCode,
        intensity: u32,
        duration: u32,
    ) -> Result<(), errors::PiShockError> {
        #[derive(Serialize, Deserialize)]
        struct PiShockAPIRequest {
            #[serde(rename(serialize = "Op"))]
            op: u32,
            #[serde(rename(serialize = "Intensity"))]
            intensity: u32,
            #[serde(rename(serialize = "Duration"))]
            duration: u32,
            #[serde(rename(serialize = "Code"))]
            sharecode: String,
            #[serde(rename(serialize = "Apikey"))]
            api_key: String,
            #[serde(rename(serialize = "Name"))]
            app_name: String,
            #[serde(rename(serialize = "Username"))]
            username: String,
        }

        debug!("Sending request to PiShock API: {{ Op: {}, Intensity: {}, Duration: {}, Code: {}, Apikey: {} }}", op_code as u32, intensity, duration, self.share_code, self.api_key);

        let http_response = self
            .http_client
            .post(self.api_server_url.clone())
            .json(&PiShockAPIRequest {
                op: op_code as u32,
                intensity,
                duration,
                sharecode: self.share_code.clone(),
                api_key: self.api_key.clone(),
                app_name: self.app_name.clone(),
                username: self.api_username.clone(),
            })
            .send()
            .await;

        return if let Ok(response) = http_response {
            debug!("Response from PiShock API: {}", response.status());
            let response_text = response.text().await;
            if let Ok(response_text) = response_text {
                error_to_pishock_error(response_text)
            } else {
                Err(errors::PiShockError::ConnectionError(
                    response_text.unwrap_err().to_string(),
                ))
            }
        } else {
            let response_code = http_response.unwrap_err().status();

            if response_code.is_some() {
                return Err(errors::PiShockError::ConnectionError(format!(
                    "Failed to connect to {}, response code: {}",
                    self.api_server_url,
                    response_code.unwrap_or(StatusCode::IM_A_TEAPOT)
                )));
            }

            Err(errors::PiShockError::ConnectionError(format!(
                "Failed to connect to {}",
                self.api_server_url
            )))
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::{PiShockController, PiShockOpCode};
    use httpmock::Method::POST;
    use httpmock::{Mock, MockServer};
    use serde_json::json;
    use test_log::test;

    fn successful_server_opcode_mock(opcode: PiShockOpCode, mock_server: &MockServer) -> Mock {
        mock_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .header("Content-Type", "application/json")
                .json_body(json!({
                    "Op": opcode as u32,
                    "Intensity": 50,
                    "Duration": 2,
                    "Code": "sharecode",
                    "Apikey": "apikey",
                    "Name": "pishock_rs",
                    "Username": "username"
                }));
            then.status(200).body("Operation Succeeded.");
        })
    }

    macro_rules! successful_opcode_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test(tokio::test)]
                async fn $name() {
                    let mockserver = httpmock::MockServer::start();

                    let pishock_controller = PiShockController::new("pishock_rs", "username", "apikey");

                    // Get a PiShocker instance without verification (we can't set the API server URL to the mock server URL yet)
                    let mut pishocker_instance = pishock_controller.get_shocker("sharecode".to_string(), false).await.unwrap();

                    // Set the API server URL to the mock server URL
                    pishocker_instance.set_api_server_url(mockserver.url(""));

                    let mock = successful_server_opcode_mock($value, &mockserver);

                    pishocker_instance.api_request($value, 50, 2).await.expect("Failed to send opcode");

                    mock.assert();
                }
            )*
        }
    }

    successful_opcode_tests! {
        test_vibrate: PiShockOpCode::Vibrate,
        test_shock: PiShockOpCode::Shock,
        test_beep: PiShockOpCode::Beep,
    }
}
