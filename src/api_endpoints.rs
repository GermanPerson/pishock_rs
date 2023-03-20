use crate::errors::error_to_pishock_error;
use crate::pishocker::PiShockerMetadata;
use crate::{errors, PiShocker};
use log::debug;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::Instant;

#[derive(Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
pub(crate) enum PiShockOpCode {
    Shock = 0,
    Vibrate = 1,
    Beep = 2,
}

impl PiShocker {
    pub(crate) async fn action_api_request(
        &self,
        op_code: PiShockOpCode,
        intensity: u32,
        duration: Duration,
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

        if self.get_shocker_online().is_some() && !self.get_shocker_online().unwrap() {
            return Err(errors::PiShockError::ShockerOffline);
        }

        if self.get_shocker_paused().is_some() && self.get_shocker_paused().unwrap() {
            return Err(errors::PiShockError::ShockerPaused);
        }

        // Check if the intensity is higher than the maximum intensity (in case we have metadata)
        if self.max_duration_error_triggered(duration).is_some() {
            return Err(errors::PiShockError::InvalidDuration(
                self.get_max_duration().unwrap().as_secs() as u32,
            ));
        }

        // Check if the duration is higher than the maximum duration (in case we have metadata)
        if self.max_intensity_error_triggered(intensity).is_some() {
            return Err(errors::PiShockError::InvalidIntensity(
                self.get_max_intensity().unwrap() as u32,
            ));
        }

        // Check shocker cooldown and return error if it is not over yet
        self.verify_shocker_cooldown()?;

        // Check for too low intensity (the API does not accept intensities below 1)
        if intensity < 1 {
            return Err(errors::PiShockError::InvalidIntensity({
                if let Some(max_intensity) = self.get_max_intensity() {
                    max_intensity as u32
                } else {
                    100
                }
            }));
        }

        if duration.as_millis() < 100 {
            return Err(errors::PiShockError::InvalidDuration({
                if let Some(max_duration) = self.get_max_duration() {
                    max_duration.as_secs() as u32
                } else {
                    15
                }
            }));
        }

        let api_duration_number: u32 = self.duration_to_pishock_api(duration);

        debug!("Sending request to PiShock API: {{ Op: {}, Intensity: {}, Duration: {}, Code: {}, Apikey: {} }}", op_code as u32, intensity, api_duration_number, self.share_code, self.api_key);

        let http_response = self
            .http_client
            .post(self.api_server_url.clone() + "/apioperate/")
            .json(&PiShockAPIRequest {
                op: op_code as u32,
                intensity,
                duration: api_duration_number,
                sharecode: self.share_code.clone(),
                api_key: self.api_key.clone(),
                app_name: self.app_name.clone(),
                username: self.api_username.clone(),
            })
            .send()
            .await;

        if let Ok(response) = http_response {
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
                    response_code.unwrap()
                )));
            }

            Err(errors::PiShockError::ConnectionError(format!(
                "Failed to connect to {}",
                self.api_server_url
            )))
        }
    }

    /// Refreshes the metadata of the given `[PiShocker]` instance.
    /// This function is called automatically when the instance is created.
    /// If you want to refresh the metadata manually, you can call this function.
    pub async fn refresh_metadata(&mut self) -> Result<(), errors::PiShockError> {
        #[derive(Serialize, Deserialize)]
        struct PiShockAPIRequest {
            #[serde(rename(serialize = "Apikey"))]
            api_key: String,
            #[serde(rename(serialize = "Username"))]
            username: String,
            #[serde(rename(serialize = "Code"))]
            sharecode: String,
        }

        debug!(
            "Request shocker metadata from PiShock API: {{ Apikey: {}, Username: {}, Code: {} }}",
            self.api_key, self.api_username, self.share_code
        );

        let http_response = self
            .http_client
            .post(self.api_server_url.clone() + "/GetShockerInfo")
            .json(&PiShockAPIRequest {
                api_key: self.api_key.clone(),
                username: self.api_username.clone(),
                sharecode: self.share_code.clone(),
            })
            .send()
            .await;

        if let Ok(response) = http_response {
            debug!("Response from PiShock API: {}", response.status());

            if response.status() != StatusCode::OK {
                return Err(errors::PiShockError::ShareCodeNotFound);
            }

            let response_metadata = response.json::<PiShockerMetadata>().await;
            if let Ok(response_text) = response_metadata {
                debug!("Response from PiShock API: {:?}", response_text);
                self.metadata = Some(response_text);
                Ok(())
            } else {
                Err(errors::PiShockError::UnknownError(
                    response_metadata.unwrap_err().to_string(),
                ))
            }
        } else {
            let response_code = http_response.unwrap_err().status();

            if response_code.is_some() {
                return Err(errors::PiShockError::ConnectionError(format!(
                    "Failed to connect to {}, response code: {}",
                    self.api_server_url.clone() + "/GetShockerInfo",
                    response_code.unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
                )));
            }

            Err(errors::PiShockError::ConnectionError(format!(
                "Failed to connect to {}",
                self.api_server_url.clone() + "/GetShockerInfo"
            )))
        }
    }

    fn verify_shocker_cooldown(&self) -> Result<(), errors::PiShockError> {
        // Lock the LastShock mutex
        let mut last_shock = self.last_shock.lock().unwrap();

        // Check that the shocker cooldown was not exceeded
        // Do a song and dance to avoid the value being moved into the if statement
        if let Some(last_shock) = last_shock.as_ref() {
            if let Some(cooldown) = self.get_shocker_cooldown() {
                if last_shock.elapsed() < cooldown {
                    return Err(errors::PiShockError::CooldownExceeded({
                        cooldown - last_shock.elapsed()
                    }));
                }
            }
        }

        // Update the last shock time
        *last_shock = Some(Instant::now());

        // Unlock the LastShock mutex
        drop(last_shock);

        Ok(())
    }

    /// The PiShock API requires the duration to be in milliseconds if it is below 1.5 seconds and in seconds if it is above 1.5 seconds.
    /// This function converts the duration to the correct format.
    /// If the duration is below 100 milliseconds, an error is returned (the API does not accept durations below 100 milliseconds).
    fn duration_to_pishock_api(&self, duration: Duration) -> u32 {
        if duration.as_secs() > 0 {
            duration.as_secs() as u32
        } else {
            duration.as_millis() as u32
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::api_endpoints::PiShockOpCode;
    use crate::errors::PiShockError;
    use crate::PiShockAccount;
    use httpmock::Method::POST;
    use httpmock::{Mock, MockServer};
    use serde_json::json;
    use std::time::Duration;
    use test_log::test;

    fn successful_server_opcode_mock(opcode: PiShockOpCode, mock_server: &MockServer) -> Mock {
        mock_server.mock(|when, then| {
            when.method(POST)
                .path("/apioperate/")
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

    fn metadata_mock(mock_server: &MockServer) -> Mock {
        mock_server.mock(|when, then| {
            when.method(POST)
                .path("/GetShockerInfo")
                .header("Content-Type", "application/json")
                .json_body(json!({
                    "Code": "sharecode",
                    "Apikey": "apikey",
                    "Username": "username"
                }));
            then.status(200).body(r#"{"clientId": 1612,"id": 2955,"name":"test 1","paused": false,"maxIntensity": 100,"maxDuration": 15,"online":true}"#);
        })
    }

    fn metadata_mock_unknown(mock_server: &MockServer) -> Mock {
        mock_server.mock(|when, then| {
            when.method(POST)
                .path("/GetShockerInfo")
                .header("Content-Type", "application/json")
                .json_body(json!({
                    "Code": "sharecode",
                    "Apikey": "apikey",
                    "Username": "username"
                }));
            then.status(404).body(r#"{  "type": "https://tools.ietf.org/html/rfc7231#section-6.5.4",  "title": "Not Found",  "status": 404,  "traceId": "00-53b12b7etdstgedstest4d0-dbcb9c3722b20ab9-00" }"#);
        })
    }

    #[test(tokio::test)]
    async fn metadata_parsing_test() {
        let mockserver = MockServer::start();

        let pishock_account = PiShockAccount::new("pishock_rs", "username", "apikey");

        // Get a PiShocker instance without verification (we can't set the API server URL to the mock server URL yet)
        let mut pishocker_instance = pishock_account
            .get_shocker_without_verification("sharecode".to_string())
            .await
            .unwrap();

        let mock = metadata_mock(&mockserver);

        // Set the API server URL to the mock server URL
        pishocker_instance.set_api_server_url(mockserver.url(""));
        pishocker_instance.refresh_metadata().await.unwrap();

        assert_eq!(pishocker_instance.get_shocker_name().unwrap(), "test 1");
        assert_eq!(pishocker_instance.get_max_intensity().unwrap(), 100);
        assert_eq!(
            pishocker_instance.get_max_duration().unwrap(),
            Duration::from_secs(15)
        );
        assert!(pishocker_instance.get_shocker_online().unwrap());
        assert!(!pishocker_instance.get_shocker_paused().unwrap());
        assert_eq!(pishocker_instance.get_shocker_id().unwrap(), 2955);

        mock.assert();
    }

    #[test(tokio::test)]
    async fn metadata_failed_request_unknown() {
        let mockserver = MockServer::start();

        let pishock_account = PiShockAccount::new("pishock_rs", "username", "apikey");

        // Get a PiShocker instance without verification (we can't set the API server URL to the mock server URL yet)
        let mut pishocker_instance = pishock_account
            .get_shocker_without_verification("sharecode".to_string())
            .await
            .unwrap();

        let mock = metadata_mock_unknown(&mockserver);

        // Set the API server URL to the mock server URL
        pishocker_instance.set_api_server_url(mockserver.url(""));

        match pishocker_instance.refresh_metadata().await {
            Ok(_) => {
                panic!("Expected error, got success");
            }
            Err(e) => match e {
                PiShockError::ShareCodeNotFound => {}
                _ => {
                    panic!("Expected ShareCodeNotFound, got {e}");
                }
            },
        }

        mock.assert();
    }

    macro_rules! successful_opcode_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test(tokio::test)]
                async fn $name() {
                    let mockserver = httpmock::MockServer::start();

                    let pishock_account = PiShockAccount::new("pishock_rs", "username", "apikey");

                    // Get a PiShocker instance without verification (we can't set the API server URL to the mock server URL yet)
                    let mut pishocker_instance = pishock_account.get_shocker_without_verification("sharecode".to_string()).await.unwrap();

                    // Set the API server URL to the mock server URL
                    pishocker_instance.set_api_server_url(mockserver.url(""));

                    let mock = successful_server_opcode_mock($value, &mockserver);

                    pishocker_instance.action_api_request($value, 50, Duration::from_secs(2)).await.expect("Failed to send opcode");

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
