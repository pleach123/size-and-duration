//! Parsing and formatting for byte sizes.
//!
//! Sizes are represented as a plain `u64` count of bytes. This module only
//! converts between that count and human-readable strings; it does not
//! define its own size newtype, so it composes with whatever the caller
//! already uses to store a byte count.

use std::fmt;

/// A unit of byte size, either decimal (powers of 1000) or binary (powers of 1024).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteUnit {
    B,
    KB,
    MB,
    GB,
    TB,
    KiB,
    MiB,
    GiB,
    TiB,
}

impl ByteUnit {
    /// Number of bytes in one unit.
    pub fn multiplier(self) -> u64 {
        match self {
            ByteUnit::B => 1,
            ByteUnit::KB => 1_000,
            ByteUnit::MB => 1_000_000,
            ByteUnit::GB => 1_000_000_000,
            ByteUnit::TB => 1_000_000_000_000,
            ByteUnit::KiB => 1 << 10,
            ByteUnit::MiB => 1 << 20,
            ByteUnit::GiB => 1 << 30,
            ByteUnit::TiB => 1 << 40,
        }
    }

    fn from_suffix(suffix: &str) -> Option<ByteUnit> {
        match suffix.to_ascii_lowercase().as_str() {
            "" | "b" => Some(ByteUnit::B),
            "kb" => Some(ByteUnit::KB),
            "mb" => Some(ByteUnit::MB),
            "gb" => Some(ByteUnit::GB),
            "tb" => Some(ByteUnit::TB),
            "kib" => Some(ByteUnit::KiB),
            "mib" => Some(ByteUnit::MiB),
            "gib" => Some(ByteUnit::GiB),
            "tib" => Some(ByteUnit::TiB),
            _ => None,
        }
    }
}

/// Why a byte size string failed to parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseByteError {
    Empty,
    InvalidNumber(String),
    UnknownUnit(String),
    Negative,
}

impl fmt::Display for ParseByteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseByteError::Empty => write!(f, "empty size string"),
            ParseByteError::InvalidNumber(s) => write!(f, "invalid number: {s:?}"),
            ParseByteError::UnknownUnit(s) => write!(f, "unknown unit: {s:?}"),
            ParseByteError::Negative => write!(f, "size cannot be negative"),
        }
    }
}

impl std::error::Error for ParseByteError {}

/// Parses a byte size string like `"1.5MB"`, `"200 KiB"`, or `"512"` (bytes).
///
/// Whitespace between the number and the unit is allowed. Unit matching is
/// case-insensitive. A missing unit is treated as plain bytes.
pub fn parse_bytes(input: &str) -> Result<u64, ParseByteError> {
    let s = input.trim();
    if s.is_empty() {
        return Err(ParseByteError::Empty);
    }

    let split_at = s
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(s.len());
    let (number_part, unit_part) = s.split_at(split_at);

    if number_part.is_empty() {
        return Err(ParseByteError::InvalidNumber(String::new()));
    }
    let value: f64 = number_part
        .parse()
        .map_err(|_| ParseByteError::InvalidNumber(number_part.to_string()))?;
    if value < 0.0 {
        return Err(ParseByteError::Negative);
    }

    let unit = ByteUnit::from_suffix(unit_part.trim())
        .ok_or_else(|| ParseByteError::UnknownUnit(unit_part.trim().to_string()))?;

    Ok((value * unit.multiplier() as f64).round() as u64)
}

/// Formats a byte count as a human-readable binary size, e.g. `"1.50 GiB"`.
///
/// Picks the largest binary unit for which the value is at least 1, and
/// shows two decimal places except for plain bytes.
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [(u64, &str); 4] = [
        (1u64 << 40, "TiB"),
        (1u64 << 30, "GiB"),
        (1u64 << 20, "MiB"),
        (1u64 << 10, "KiB"),
    ];

    for (threshold, name) in UNITS {
        if bytes >= threshold {
            return format!("{:.2} {}", bytes as f64 / threshold as f64, name);
        }
    }
    format!("{bytes} B")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_bytes() {
        assert_eq!(parse_bytes("512").unwrap(), 512);
        assert_eq!(parse_bytes("512B").unwrap(), 512);
    }

    #[test]
    fn parses_decimal_units() {
        assert_eq!(parse_bytes("1.5MB").unwrap(), 1_500_000);
        assert_eq!(parse_bytes("2GB").unwrap(), 2_000_000_000);
    }

    #[test]
    fn parses_binary_units_with_whitespace() {
        assert_eq!(parse_bytes("200 KiB").unwrap(), 200 * 1024);
    }

    #[test]
    fn rejects_unknown_unit() {
        assert!(matches!(
            parse_bytes("10 xb"),
            Err(ParseByteError::UnknownUnit(_))
        ));
    }

    #[test]
    fn rejects_negative_and_empty() {
        assert_eq!(parse_bytes(""), Err(ParseByteError::Empty));
        assert!(matches!(
            parse_bytes("-5MB"),
            Err(ParseByteError::InvalidNumber(_))
        ));
    }

    #[test]
    fn formats_across_ranges() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1536), "1.50 KiB");
        assert_eq!(format_bytes(3 * (1u64 << 30)), "3.00 GiB");
    }
}
