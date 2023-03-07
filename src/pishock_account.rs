use crate::{errors, PiShocker};

/// A struct representing PiShock account credentials.
/// Should be used to create [`PiShocker`] instances.
///
/// Construct a new instance with [`PiShockAccount::new`].
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PiShockAccount {
    app_name: String,
    api_username: String,
    api_key: String,
}

impl PiShockAccount {
    #[must_use]
    pub fn new<S: Into<String>>(api_name: S, api_username: S, api_key: S) -> PiShockAccount {
        PiShockAccount {
            app_name: api_name.into(),
            api_username: api_username.into(),
            api_key: api_key.into(),
        }
    }

    /// Returns a [`PiShocker`] instance for the specified share code
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// # use pishock_rs::PiShockAccount;
    /// let pishock_account = PiShockAccount::new("pishock_rs", "username", "apikey");
    /// let pishocker_instance = pishock_account.get_shocker("sharecode").await;
    /// # });
    /// ```
    /// # Errors
    /// Will return an error if shocker cannot be connected to (does NOT fail for paused shockers).
    pub async fn get_shocker<S: Into<String>>(
        &self,
        share_code: S,
    ) -> Result<PiShocker, errors::PiShockError> {
        let mut pishock_instance = self.get_shocker_without_verification(share_code).await?;

        // Fetch metadata of the shocker
        pishock_instance.refresh_metadata().await?;

        // Check if the shocker is online
        if pishock_instance.get_shocker_online().is_some()
            && pishock_instance.get_shocker_online().unwrap_or(false)
        {
            return Err(errors::PiShockError::ShockerOffline);
        }

        Ok(pishock_instance)
    }

    /// Returns a [`PiShocker`] instance for the specified share code
    /// without verifying the shocker is online and without fetching any metadata.
    pub async fn get_shocker_without_verification<S: Into<String>>(
        &self,
        share_code: S,
    ) -> Result<PiShocker, errors::PiShockError> {
        let pishock_instance = PiShocker::new(
            share_code.into(),
            self.api_key.clone(),
            self.api_username.clone(),
            self.app_name.clone(),
        );

        Ok(pishock_instance)
    }
}
