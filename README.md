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

```
$ tcping google.com:443

> 172.217.167.110:443 (warmup): 6.37 ms
> 172.217.167.110:443: 7.09 ms
> 172.217.167.110:443: 5.84 ms
> 172.217.167.110:443: 7.20 ms
> 172.217.167.110:443: 6.00 ms

  Sent = 4, Received = 4 (100%)
  Minimum = 5.84ms, Maximum = 7.20ms, Average = 6.53ms
```

Continuous monitoring (timestamps) with a 500ms interval:

```
$ tcping google.com:443 -t --interval 500
[01:21:31] 172.217.167.110:443 (warmup): 6.19 ms
[01:21:31] 172.217.167.110:443: 6.94 ms
[01:21:32] 172.217.167.110:443: 6.88 ms
[01:21:32] 172.217.167.110:443: 6.47 ms
...
```

Service availability monitoring (when ICMP ping is blocked):

```
$ tcping myserver.myregion.cloudapp.azure.com:22 -t
[01:12:15] XXX.XXX.XXX.XXX:22 (warmup): connection timed out
[01:12:20] XXX.XXX.XXX.XXX:22: connection timed out
[01:12:25] XXX.XXX.XXX.XXX:22: connection timed out
[01:12:30] XXX.XXX.XXX.XXX:22: 551.16 ms
[01:12:32] XXX.XXX.XXX.XXX:22: 14.24 ms
```

## Tips

- If you just run `tcping` without arguments, the help message is printed.
- Use `-n` to send a fixed number of timed requests (in addition to the warmup request).
- Use smaller `-i` values for tighter sampling in continuous mode.

---

See `tcping --help` for all options and usage tips.
