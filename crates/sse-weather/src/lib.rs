//! SSE4 weather (track W, weather design): convincing skies from rules, behind
//! one contract a real simulation can replace without breaking saves.
//!
//! - [`contract`]: the `WeatherField` every reader uses ([`WeatherBlock`]).
//! - [`conditions`]: the facts a block reads, keyed on conditions, never on kind.
//! - [`circulation`]: W1, the circulation regime, what condenses, climate averages.
//! - [`rules`]: the rule-based block, `sse.weather.rules`.

pub mod aerosols;
pub mod circulation;
pub mod conditions;
pub mod contract;
pub mod formations;
pub mod giants;
pub mod hooks;
pub mod rules;
pub mod scenario;
pub mod settings;
pub mod swap;
pub mod synoptic;
pub mod zonal;

pub use conditions::Conditions;
pub use contract::*;
pub use rules::RuleBlock;
pub use settings::WeatherSettings;

pub const WEATHER_VERSION: &str = "weather-v1";

#[cfg(test)]
mod tests;
