use log::debug;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum PiShockError {
    #[error("Share code doesn't exist")]
    ShareCodeNotFound,
    #[error("Username or API key invalid")]
    InvalidCredentials,
    #[error("Shocker is in paused state")]
    ShockerPaused,
    #[error("Shocker is offline")]
    ShockerOffline,
    #[error("Share code is already in use")]
    ShareCodeInUse,
    #[error("Invalid OP code specified: {}", .0)]
    InvalidOpCode(u32),
    #[error("Invalid intensity specified, max intensity: {}", .0)]
    InvalidIntensity(u32),
    #[error("Invalid duration specified: {}", .0)]
    InvalidDuration(u32),
    #[error("Connection error: {}", .0)]
    ConnectionError(String),
    #[error("Unknown error: {}", .0)]
    UnknownError(String),
}

/// Converts possible HTTP responses to the respective `PiShock` errors
/// This list is NOT exhaustive, the skipped errors are handled by the `PiShock` functions and should not ever be sent by the API
pub(crate) fn error_to_pishock_error<S: Into<String> + Clone>(error: S) -> Result<(), PiShockError> {
    debug!("Resolving body text to PiShockError: {}", error.clone().into());

    if error.clone().into().contains("Intensity must be between 0 and ") {
        return Err(PiShockError::InvalidIntensity(error.into().split(' ').last().unwrap().parse().unwrap()));
    }

    match error.clone().into().as_ref() {
        "Operation Succeeded." => Ok(()),
        "Share code not found" | "This code doesn’t exist." => Err(PiShockError::ShareCodeNotFound),
        "Not Authorized." => Err(PiShockError::InvalidCredentials),
        "Shocker is Paused, unable to send command." => Err(PiShockError::ShockerPaused),
        "Device currently not connected." => Err(PiShockError::ShockerOffline),
        "This share code has already been used by somebody else." => Err(PiShockError::ShareCodeInUse),
        _ => Err(PiShockError::UnknownError(error.into())),
    }
}
