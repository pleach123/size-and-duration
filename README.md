# size-and-duration

Parsing and formatting for two things that show up constantly in config
files, logs, and CLI flags: byte sizes (`"512MB"`, `"2GiB"`) and durations
(`"30s"`, `"1h30m"`). Every function is a pure conversion between a plain
number and a string — no I/O, no global state — so they're trivial to unit
test and safe to call from anywhere.

## Why

Most projects end up writing their own ad hoc parser for a `--max-size
500MB` or `--timeout 30s` flag, usually as a one-off regex buried in
argument-parsing code. This crate pulls that logic out into small,
well-tested functions that take a string and return a `u64` or a
`Duration`, or the reverse.

## Usage

```rust
use size_and_duration::{format_bytes, format_duration, parse_bytes, parse_duration};

let max_upload = parse_bytes("500MB").unwrap();
assert_eq!(max_upload, 500_000_000);

let cache_size = parse_bytes("2 GiB").unwrap();
assert_eq!(format_bytes(cache_size), "2.00 GiB");

let timeout = parse_duration("1h30m").unwrap();
assert_eq!(timeout.as_secs(), 5400);
assert_eq!(format_duration(timeout), "1h30m");
```

## Byte sizes

`parse_bytes` accepts a number followed by an optional unit: `B`, `KB`,
`MB`, `GB`, `TB` (decimal, powers of 1000) or `KiB`, `MiB`, `GiB`, `TiB`
(binary, powers of 1024). No unit means plain bytes. `format_bytes` always
renders using the binary units, since that's what most people reading a
memory or disk figure actually expect.

## Durations

`parse_duration` accepts one or more `<number><unit>` pairs chained
together, e.g. `"2d12h"` or `"90s"`. Recognized units: `d`, `h`, `m`, `s`,
`ms`, `us`, `ns`. `format_duration` renders the same style back, using the
largest units that divide evenly.

## Status

Early skeleton — two modules, no cross-cutting types yet. See commit
history for what's coming next.

## License

MIT
