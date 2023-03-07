mod api_endpoints;
pub mod errors;
mod pishocker;
pub use self::pishocker::*;
mod pishock_account;
pub use self::pishock_account::*;

/// The base URL for the PiShock API (without trailing slash)
static PUBLIC_PISHOCK_API_BASE: &str = "https://do.pishock.com/api";
