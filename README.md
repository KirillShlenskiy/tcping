# tcping

Cross-platform TCP connect monitoring and latency measurement tool, similar to [PsPing](https://learn.microsoft.com/en-us/sysinternals/downloads/psping), with colored console output and a simple summary.

`tcping` is useful when you care about service reachability on a specific port, especially in environments where ICMP ping is blocked or not representative of real application traffic.

## Features

- Measures TCP connect latency to a `host:port`
- Performs one warmup connect before timed probes
- Excludes the warmup connect from summary statistics
- Reports sent/received counts plus min/max/avg latency
- Supports continuous monitoring with timestamps via `-t`
- Lets you control probe interval in milliseconds via `-i`

## Build

Build locally from source:

- Debug: `cargo build`
- Release: `cargo build --release`

Binary paths:

- macOS / Linux: `target/{debug,release}/tcping`
- Windows: `target\{debug,release}\tcping.exe`

Run without installing:

```sh
cargo run -- google.com:443
```

## Quick Start

```sh
# Basic connectivity and latency check
tcping google.com:443

# Send 10 timed probes (plus one warmup probe)
tcping -n 10 google.com:443

# Monitor continuously with timestamps every 500 ms
tcping -t -i 500 google.com:443
```

## Usage

```text
$ tcping --help
TCP ping utility by Kirill Shlenskiy (2024) v0.9.10

Usage: tcping [OPTIONS] <target>

Arguments:
  <target>  TCP ping target in "host:port" format (i.e. google.com:80)

Options:
  -t, --continuous           Ping until stopped with Ctrl+C
  -n, --count <count>        Number of TCP requests (not counting warmup) to send [default: 4]
  -i, --interval <interval>  Interval (in milliseconds) between requests [default: 1000]
  -h, --help                 Print help
```

## Behavior Notes

- `tcping` performs TCP connects, not ICMP echo requests.
- The target must be passed as `host:port`.
- Default TCP connect timeout is 4 seconds.
- A single warmup probe is always sent first and is not included in the summary.
- In continuous mode, probing continues until you stop it with `Ctrl+C`.
- Summary colors indicate success ratio: green for `100%`, red for `0%`, yellow otherwise.

## Examples

Basic port connectivity testing:

```sh
tcping google.com:443
```

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/screens/basic_example.svg" />
  <img alt="Example tcping output" src="docs/screens/basic_example.svg" />
</picture>

Continuous monitoring with timestamps and a 500 ms interval:

```sh
tcping -t -i 500 google.com:443
```

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/screens/continuous_example.svg" />
  <img alt="Continuous tcping output" src="docs/screens/continuous_example.svg" />
</picture>

Service availability monitoring for a specific port when ICMP ping is blocked:

```sh
tcping -t db.internal.example:5432
```

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/screens/service_example.svg" />
  <img alt="Service availability tcping output" src="docs/screens/service_example.svg" />
</picture>

## Tips

- Running `tcping` without arguments prints the help message.
- `-n` controls timed probes only; the warmup probe is extra.
- Smaller `-i` values give tighter sampling but produce more connection attempts.

See `tcping --help` for the current option list.
