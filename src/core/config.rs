use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const CONFIG_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    pub sources: Vec<PathBuf>,
    pub destination: Destination,
    pub schedule: Schedule,
    pub excludes: Vec<String>,
    pub versioning: Versioning,
    pub mirror: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            sources: Vec::new(),
            destination: Destination {
                drive_id: String::new(),
                path: PathBuf::new(),
            },
            schedule: Schedule::Manual,
            excludes: Vec::new(),
            versioning: Versioning {
                enabled: true,
                retention: RetentionPolicy::KeepLast { count: 30 },
            },
            mirror: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Destination {
    pub drive_id: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Schedule {
    Manual,
    Interval { minutes: u32 },
    DailyAt { hour: u8, minute: u8 },
    WeeklyAt { weekday: Weekday, hour: u8, minute: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Weekday {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Versioning {
    pub enabled: bool,
    pub retention: RetentionPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RetentionPolicy {
    KeepLast { count: u32 },
    KeepDays { days: u32 },
    Tiered { daily_days: u32, weekly_weeks: u32, monthly_months: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    UnsupportedVersion(u32),
    InvalidScheduleTime,
    EmptySources,
    EmptyDestinationDrive,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::UnsupportedVersion(v) => write!(f, "unsupported config version: {v}"),
            ConfigError::InvalidScheduleTime => write!(f, "invalid schedule time"),
            ConfigError::EmptySources => write!(f, "config has no sources"),
            ConfigError::EmptyDestinationDrive => write!(f, "config destination drive is empty"),
        }
    }
}

impl std::error::Error for ConfigError {}

pub fn validate(config: &Config) -> Result<(), ConfigError> {
    if config.version != CONFIG_VERSION {
        return Err(ConfigError::UnsupportedVersion(config.version));
    }
    if config.sources.is_empty() {
        return Err(ConfigError::EmptySources);
    }
    if config.destination.drive_id.is_empty() {
        return Err(ConfigError::EmptyDestinationDrive);
    }
    match config.schedule {
        Schedule::Manual | Schedule::Interval { .. } => {}
        Schedule::DailyAt { hour, minute } | Schedule::WeeklyAt { hour, minute, .. } => {
            if hour >= 24 || minute >= 60 {
                return Err(ConfigError::InvalidScheduleTime);
            }
        }
    }
    Ok(())
}

pub fn to_json(config: &Config) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(config)
}

pub fn from_json(json: &str) -> Result<Config, serde_json::Error> {
    serde_json::from_str(json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn arb_weekday() -> impl Strategy<Value = Weekday> {
        prop_oneof![
            Just(Weekday::Mon),
            Just(Weekday::Tue),
            Just(Weekday::Wed),
            Just(Weekday::Thu),
            Just(Weekday::Fri),
            Just(Weekday::Sat),
            Just(Weekday::Sun),
        ]
    }

    fn arb_schedule() -> impl Strategy<Value = Schedule> {
        prop_oneof![
            Just(Schedule::Manual),
            (1u32..10_000).prop_map(|minutes| Schedule::Interval { minutes }),
            (0u8..24, 0u8..60).prop_map(|(hour, minute)| Schedule::DailyAt { hour, minute }),
            (arb_weekday(), 0u8..24, 0u8..60)
                .prop_map(|(weekday, hour, minute)| Schedule::WeeklyAt { weekday, hour, minute }),
        ]
    }

    fn arb_retention() -> impl Strategy<Value = RetentionPolicy> {
        prop_oneof![
            (1u32..1000).prop_map(|count| RetentionPolicy::KeepLast { count }),
            (1u32..3650).prop_map(|days| RetentionPolicy::KeepDays { days }),
            (0u32..365, 0u32..52, 0u32..120).prop_map(|(d, w, m)| RetentionPolicy::Tiered {
                daily_days: d,
                weekly_weeks: w,
                monthly_months: m,
            }),
        ]
    }

    fn arb_config() -> impl Strategy<Value = Config> {
        (
            prop::collection::vec("[a-zA-Z0-9_/.-]{1,40}", 0..5),
            "[a-zA-Z0-9_-]{1,30}",
            "[a-zA-Z0-9_/.-]{1,40}",
            arb_schedule(),
            prop::collection::vec("[a-zA-Z0-9*?._-]{1,20}", 0..5),
            any::<bool>(),
            arb_retention(),
            any::<bool>(),
        )
            .prop_map(
                |(sources, drive_id, dest_path, schedule, excludes, ver_enabled, retention, mirror)| {
                    Config {
                        version: CONFIG_VERSION,
                        sources: sources.into_iter().map(PathBuf::from).collect(),
                        destination: Destination {
                            drive_id,
                            path: PathBuf::from(dest_path),
                        },
                        schedule,
                        excludes,
                        versioning: Versioning { enabled: ver_enabled, retention },
                        mirror,
                    }
                },
            )
    }

    proptest! {
        #[test]
        fn round_trip_json(config in arb_config()) {
            let json = to_json(&config).expect("serialize");
            let parsed = from_json(&json).expect("deserialize");
            prop_assert_eq!(config, parsed);
        }
    }

    #[test]
    fn default_config_validates_after_setting_required_fields() {
        let mut c = Config::default();
        c.sources.push(PathBuf::from("/home/alvin/docs"));
        c.destination.drive_id = "drive-123".into();
        validate(&c).unwrap();
    }

    #[test]
    fn rejects_unsupported_version() {
        let mut c = Config::default();
        c.sources.push(PathBuf::from("/x"));
        c.destination.drive_id = "d".into();
        c.version = 999;
        assert_eq!(validate(&c), Err(ConfigError::UnsupportedVersion(999)));
    }

    #[test]
    fn rejects_invalid_schedule_time() {
        let mut c = Config::default();
        c.sources.push(PathBuf::from("/x"));
        c.destination.drive_id = "d".into();
        c.schedule = Schedule::DailyAt { hour: 25, minute: 0 };
        assert_eq!(validate(&c), Err(ConfigError::InvalidScheduleTime));
    }

    #[test]
    fn rejects_empty_sources() {
        let mut c = Config::default();
        c.destination.drive_id = "d".into();
        assert_eq!(validate(&c), Err(ConfigError::EmptySources));
    }
}
