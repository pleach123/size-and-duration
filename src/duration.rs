//! Parsing and formatting for durations.
//!
//! Durations are represented with `std::time::Duration`; this module only
//! converts between that type and compact human-readable strings such as
//! `"1h30m"` or `"250ms"`.

use std::fmt;
use std::time::Duration;

/// Why a duration string failed to parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseDurationError {
    Empty,
    InvalidNumber(String),
    UnknownUnit(String),
}

impl fmt::Display for ParseDurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseDurationError::Empty => write!(f, "empty duration string"),
            ParseDurationError::InvalidNumber(s) => write!(f, "invalid number: {s:?}"),
            ParseDurationError::UnknownUnit(s) => write!(f, "unknown unit: {s:?}"),
        }
    }
}

impl std::error::Error for ParseDurationError {}

fn nanos_per_unit(unit: &str) -> Option<f64> {
    match unit {
        "d" => Some(86_400_000_000_000.0),
        "h" => Some(3_600_000_000_000.0),
        "m" => Some(60_000_000_000.0),
        "s" => Some(1_000_000_000.0),
        "ms" => Some(1_000_000.0),
        "us" => Some(1_000.0),
        "ns" => Some(1.0),
        _ => None,
    }
}

/// Parses a duration string made of `<number><unit>` pairs, e.g. `"1h30m"`,
/// `"90s"`, or `"2d12h"`. Recognized units: `d`, `h`, `m`, `s`, `ms`, `us`,
/// `ns`. Pairs may be chained without separators; each number may be
/// fractional.
pub fn parse_duration(input: &str) -> Result<Duration, ParseDurationError> {
    let s = input.trim();
    if s.is_empty() {
        return Err(ParseDurationError::Empty);
    }

    let mut total_nanos: f64 = 0.0;
    let mut rest = s;

    while !rest.is_empty() {
        let digits_end = rest
            .find(|c: char| !(c.is_ascii_digit() || c == '.'))
            .unwrap_or(rest.len());
        if digits_end == 0 {
            return Err(ParseDurationError::InvalidNumber(rest.to_string()));
        }
        let (num_str, after_num) = rest.split_at(digits_end);
        let value: f64 = num_str
            .parse()
            .map_err(|_| ParseDurationError::InvalidNumber(num_str.to_string()))?;

        let unit_end = after_num
            .find(|c: char| c.is_ascii_digit())
            .unwrap_or(after_num.len());
        let (unit_str, next) = after_num.split_at(unit_end);
        let per_unit = nanos_per_unit(unit_str)
            .ok_or_else(|| ParseDurationError::UnknownUnit(unit_str.to_string()))?;

        total_nanos += value * per_unit;
        rest = next;
    }

    Ok(Duration::from_nanos(total_nanos.round() as u64))
}

/// Formats a duration as a compact string using the largest units that fit,
/// e.g. `Duration::from_secs(5400)` becomes `"1h30m"`.
pub fn format_duration(duration: Duration) -> String {
    let total_nanos = duration.as_nanos();
    if total_nanos == 0 {
        return "0s".to_string();
    }

    const UNITS: [(u128, &str); 7] = [
        (86_400_000_000_000, "d"),
        (3_600_000_000_000, "h"),
        (60_000_000_000, "m"),
        (1_000_000_000, "s"),
        (1_000_000, "ms"),
        (1_000, "us"),
        (1, "ns"),
    ];

    let mut remaining = total_nanos;
    let mut out = String::new();
    for (unit_nanos, name) in UNITS {
        let count = remaining / unit_nanos;
        if count > 0 {
            out.push_str(&count.to_string());
            out.push_str(name);
            remaining %= unit_nanos;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_unit() {
        assert_eq!(parse_duration("90s").unwrap(), Duration::from_secs(90));
        assert_eq!(parse_duration("250ms").unwrap(), Duration::from_millis(250));
    }

    #[test]
    fn parses_chained_units() {
        assert_eq!(
            parse_duration("1h30m").unwrap(),
            Duration::from_secs(3600 + 30 * 60)
        );
        assert_eq!(
            parse_duration("2d12h").unwrap(),
            Duration::from_secs(2 * 86_400 + 12 * 3600)
        );
    }

    #[test]
    fn rejects_unknown_unit_and_empty() {
        assert_eq!(parse_duration(""), Err(ParseDurationError::Empty));
        assert!(matches!(
            parse_duration("5x"),
            Err(ParseDurationError::UnknownUnit(_))
        ));
    }

    #[test]
    fn formats_compactly() {
        assert_eq!(format_duration(Duration::from_secs(5400)), "1h30m");
        assert_eq!(format_duration(Duration::from_millis(250)), "250ms");
        assert_eq!(format_duration(Duration::ZERO), "0s");
    }

    #[test]
    fn round_trips_through_parse_and_format() {
        let d = Duration::from_secs(90_061);
        let formatted = format_duration(d);
        let parsed = parse_duration(&formatted).unwrap();
        assert_eq!(parsed, d);
    }
}
