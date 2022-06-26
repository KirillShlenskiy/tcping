# tcping

Cross-platform TCP Connect monitoring and latency measurement utility functionally similar to [PsPing](https://docs.microsoft.com/en-us/sysinternals/downloads/psping), but with coloured console output.

## Common use cases

Basic port connectivity testing:

```
$ tcping google.com:443

> 142.250.70.174:443 (warmup): 30.92 ms
> 142.250.70.174:443: 24.26 ms
> 142.250.70.174:443: 24.25 ms
> 142.250.70.174:443: 23.94 ms
> 142.250.70.174:443: 25.70 ms

  Sent = 4, Received = 4 (100%)
  Minimum = 23.94ms, Maximum = 25.70ms, Average = 24.54ms
```

Server reboot / service availability monitoring (when ICMP ping is not an option - i.e. cloud scenarios):

```
$ tcping myserver.myregion.cloudapp.azure.com:22 -t

[01:12:15] XXX.XXX.XXX.XXX:22 (warmup): connection timed out
[01:12:20] XXX.XXX.XXX.XXX:22: connection timed out
[01:12:25] XXX.XXX.XXX.XXX:22: connection timed out
[01:12:30] XXX.XXX.XXX.XXX:22: 551.16 ms
[01:12:32] XXX.XXX.XXX.XXX:22: 14.24 ms
[01:12:33] XXX.XXX.XXX.XXX:22: 14.85 ms
[01:12:34] XXX.XXX.XXX.XXX:22: 14.70 ms

...
```

See `tcping --help` for additional command-line options and usage tips.
