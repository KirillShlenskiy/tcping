# tcping

Cross-platform TCP connect monitoring and latency measurement tool, similar to [PsPing](https://docs.microsoft.com/en-us/sysinternals/downloads/psping) — with colored console output and simple stats.

## Features

- Warmup connect followed by timed connects (warmup is not counted in stats)
- Success ratio and min/max/avg latency summary
- Continuous monitoring mode with timestamps (`-t`)
- Adjustable interval between requests (`-i`, milliseconds)

## Installation / Build

- From source (local build):
  - Debug: `cargo build` → `target/debug/tcping(.exe)`
  - Release: `cargo build --release` → `target/release/tcping(.exe)`

## Usage

```
$ tcping --help
TCP ping utility by Kirill Shlenskiy (2024) v0.9.10

Usage: tcping[.exe] [OPTIONS] <target>

Arguments:
  <target>  TCP ping target in "host:port" format (i.e. google.com:80)

Options:
  -t, --continuous    Ping until stopped with Ctrl+C
  -n, --count <n>     Number of TCP requests (not counting warmup) to send
  -i, --interval <i>  Interval (in milliseconds) between requests; the default is 1000
  -h, --help          Print help
```

Notes:
- Default TCP connect timeout is 4 seconds.
- Colors indicate success ratio (green for 100%, red for 0%, yellow otherwise).

## Examples

Basic port connectivity testing:

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/screens/basic_example.svg" />
  <img alt="Example tcping output" src="docs/screens/basic_example.svg" />
</picture>

Continuous monitoring (timestamps) with a 500ms interval:

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/screens/continuous_example.svg" />
  <img alt="Continuous tcping output" src="docs/screens/continuous_example.svg" />
</picture>

Service availability monitoring (when ICMP ping is blocked):

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/screens/service_example.svg" />
  <img alt="Service availability tcping output" src="docs/screens/service_example.svg" />
</picture>

## Tips

- If you just run `tcping` without arguments, the help message is printed.
- Use `-n` to send a fixed number of timed requests (in addition to the warmup request).
- Use smaller `-i` values for tighter sampling in continuous mode.

---

See `tcping --help` for all options and usage tips.
