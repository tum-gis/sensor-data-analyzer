use chrono::Utc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DurationParseError {
    #[error("failed to parse human-readable duration: {0}")]
    HumanTimeError(#[from] humantime::DurationError),

    #[error("failed to convert to chrono::Duration: {0}")]
    ChronoConversionError(#[from] chrono::OutOfRangeError),
}

pub fn parse_duration(arg: &str) -> Result<chrono::Duration, DurationParseError> {
    let std_duration: std::time::Duration = arg.parse::<humantime::Duration>().map(Into::into)?;
    let chrono_duration = chrono::Duration::from_std(std_duration)?;
    Ok(chrono_duration)
}

#[derive(Debug, Error)]
pub enum TimestampParseError {
    #[error("failed to convert to chrono::DateTime<Utc>: {0}")]
    ChronoConversionError(#[from] chrono::ParseError),
}

// see: https://stackoverflow.com/a/61179071
pub fn parse_timestamp(arg: &str) -> Result<chrono::DateTime<Utc>, TimestampParseError> {
    let chrono_datetime: chrono::DateTime<Utc> =
        chrono::DateTime::parse_from_str(arg, "%Y-%m-%d %H:%M:%S%.9f %z")?.into();
    Ok(chrono_datetime)
}
