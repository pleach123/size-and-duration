//! Conversions and formatting for byte sizes and durations.
//!
//! Every public function here is a pure conversion: string in, number out,
//! or the reverse. Nothing touches the filesystem, the clock, or any other
//! ambient state, which is what keeps them cheap to unit test.

pub mod bytes;
pub mod duration;

pub use bytes::{format_bytes, parse_bytes, ByteUnit, ParseByteError};
pub use duration::{format_duration, parse_duration, ParseDurationError};
