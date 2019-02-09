extern crate clap;
extern crate console;

use std::error::Error;
use std::io;
use std::io::prelude::*;
use std::net::{SocketAddr,ToSocketAddrs,TcpStream};
use std::time::{Duration,Instant};
use std::thread;

use crate::clap::{App,AppSettings,Arg};
use crate::console::style;

mod aggregates;

fn main() {
    let matches = App::new("tcping")
        .version("0.3")
        .about("TCP ping utility by Kirill Shlenskiy (2019)")
        .arg(Arg::from_usage("<target> 'TCP ping target in {host:port} format (i.e. google.com:80)'"))
        .arg(Arg::from_usage("-c, --count=[count] 'Number of requests (not counting warmup) to dispatch'"))
        .arg(Arg::from_usage("-i, --interval=[interval] 'Interval (in milliseconds) between requests; the default is 1000'"))
        .arg(Arg::from_usage("-t, --timeout=[timeout] 'Connection timeout in seconds; the default is 4'"))
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::DisableVersion)
        .get_matches();

    let count = matches
        .value_of("count")
        .map(|c| c.parse::<u64>().expect("Invalid count."))
        .unwrap_or(4);

    let interval_ms = matches
        .value_of("interval")
        .map(|c| c.parse::<u64>().expect("Invalid interval."))
        .unwrap_or(1_000);

    let timeout_secs = matches
        .value_of("timeout")
        .map(|c| c.parse::<u64>().expect("Invalid timeout."))
        .unwrap_or(4);

    let target = matches.value_of("target").unwrap();
    let socket_addr_result = target.to_socket_addrs();

    if let Err(err) = socket_addr_result {
        let error_text = {
            if format!("{}", err) == "invalid socket address" {
                String::from("Invalid argument. Expected format: 'host:port' (i.e. 'google.com:80').")
            }
            else {
                fmt_err(&err)
            }
        };

        println!("{}", error_text);
        return;
    }

    let addr = socket_addr_result.unwrap().next().unwrap();

    // Warmup.
    print_timed_ping(&addr, timeout_secs, true).ok();

    // Actual timed ping.
    let mut results = Vec::new();

    for _i in 0..count {
        thread::sleep(Duration::from_millis(interval_ms));
        results.push(print_timed_ping(&addr, timeout_secs, false).ok());
    }

    // Print stats (psping format):
    println!();
    print_stats(results);
}

fn print_timed_ping(addr: &SocketAddr, timeout_secs: u64, warmup: bool) -> Result<f64, std::io::Error> {
    if warmup {
        print!("> {} (warmup): ", addr);
        io::stdout().flush().unwrap();
    }
    else {
        print!("> {}: ", addr);
    }

    match timed_ping(&addr, timeout_secs) {
        Err(err) => {
            println!("{}", style(&err).red());
            Err(err)
        }
        Ok(latency_ms) => {
            println!("{:.2} ms", style(latency_ms).green());
            Ok(latency_ms)
        }
    }
}

fn timed_ping(addr: &SocketAddr, timeout_secs: u64) -> Result<f64, std::io::Error> {
    let start = Instant::now();

    if let Err(err) = TcpStream::connect_timeout(&addr, Duration::from_secs(timeout_secs)) {
        return Err(err);
    }

    let finish = Instant::now();
    let diff = finish - start;
    let diff_ns = diff.subsec_nanos();
    let diff_ms = diff_ns as f64 / 1_000_000 as f64 + diff.as_secs() as f64 * 1_000 as f64;

    Ok(diff_ms)
}

fn print_stats(results: Vec<Option<f64>>) {
    let successes: Vec<f64> = results.iter().filter(|l| l.is_some()).map(|l| l.unwrap()).collect();
    let success_percent = successes.len() * 100 / results.len();

    let formatted_percent = {
        if success_percent == 100 {
            format!("{}{}", style(&success_percent).green(), style("%").green())
        }
        else {
            format!("{}{}", style(&success_percent).red(), style("%").red())
        }
    };

    println!("  Sent = {:.2}, Received = {:.2} ({})", results.len(), successes.len(), formatted_percent);

    if !results.is_empty() {
        println!(
            "  Minimum = {:.2}ms, Maximum = {:.2}ms, Average = {:.2}ms",
            aggregates::min(&successes),
            aggregates::max(&successes),
            aggregates::avg(&successes)
        );
    }
}

fn fmt_err(err: &Error) -> String {
    let mut desc = Vec::new();
    {
        let desc_orig = format!("{}", err);
        let mut desc_chars = desc_orig.chars();

        while let Some(c) = desc_chars.next() {
            // Capitalise first letter.
            if desc.is_empty() && c.is_lowercase() {
                for u in c.to_uppercase() {
                    desc.push(u);
                }
            }
            else {
                desc.push(c);
            }
        }
    }

    // Ensure there is a full stop at the end.
    if let Some(last_char) = desc.last() {
        if last_char.to_owned() != '.' {
            desc.push('.');
        }
    }

    // Collect Vec<char> -> String.
    desc.into_iter().collect()
}