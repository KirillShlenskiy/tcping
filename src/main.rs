extern crate clap;
extern crate console;

use std::error::Error;
use std::io::{self, Error as IOError, Write};
use std::net::{SocketAddr, ToSocketAddrs, TcpStream};
use std::str::FromStr;
use std::thread;
use std::time::{Duration, Instant};

use crate::clap::{App, AppSettings, Arg, ArgMatches};
use crate::console::style;

mod aggregates;

const TIMEOUT_SECS: u64 = 4;

fn main() -> Result<(), Box<Error>> {
    let matches = App::new("tcping")
        .version("0.5.0")
        .about("TCP ping utility by Kirill Shlenskiy (2019)")
        .arg(Arg::from_usage("<target> 'TCP ping target in \"host:port\" format (i.e. google.com:80)'"))
        .arg(Arg::from_usage("-t 'Ping until stopped with Ctrl+C'"))
        .arg(Arg::from_usage("-n=[count] 'Number of TCP requests (not counting warmup) to send'"))
        .arg(Arg::from_usage("-i=[interval] 'Interval (in milliseconds) between requests; the default is 1000'"))
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::DisableVersion)
        .setting(AppSettings::UnifiedHelpMessage)
        .get_matches();

    let continuous = matches.is_present("t");
    let count: u64 = parse_arg(&matches, "n", 4)?;
    let interval_ms: u64 = parse_arg(&matches, "i", 1_000)?;
    let target = matches.value_of("target").unwrap();

    let addr = match target.to_socket_addrs() {
        Ok(mut addr_list) => addr_list.next().unwrap(),
        Err(err) => {
            let error_text = {
                if format!("{}", err) == "invalid socket address" {
                    String::from("Invalid argument. Expected format: 'host:port' (i.e. 'google.com:80').")
                }
                else {
                    fmt_err(&err)
                }
            };

            println!("{}", error_text);
            return Err(Box::new(err));
        }
    };

    // Warmup.
    print_timed_ping(&addr, TIMEOUT_SECS, true).ok();

    // Actual timed ping.
    let mut results = Vec::new();

    loop {
        if !continuous && results.len() as u64 >= count {
            break;
        }

        thread::sleep(Duration::from_millis(interval_ms));
        results.push(print_timed_ping(&addr, TIMEOUT_SECS, false).ok());
    }

    if !results.is_empty() {
        // Print stats (psping format):
        println!();
        print_stats(&results);
    }
    
    Ok(())
}

fn parse_arg<T : FromStr>(matches: &ArgMatches, name: &str, default_value: T) -> Result<T, T::Err> {
    match matches.value_of(name) {
        None => Ok(default_value),
        Some(value_str) => {
            match value_str.parse() {
                Ok(value) => Ok(value),
                Err(err) => {
                    println!("Invalid {}.", name);
                    Err(err)
                }
            }
        }
    }
}

fn print_timed_ping(addr: &SocketAddr, timeout_secs: u64, warmup: bool) -> Result<f64, IOError> {
    if warmup {
        print!("> {} (warmup): ", addr);
        io::stdout().flush()?;
    }
    else {
        print!("> {}: ", addr);
    }

    match timed_ping(&addr, timeout_secs) {
        Err(err) => {
            println!("{}", style(&err).cyan());
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

fn print_stats(results: &[Option<f64>]) {
    let successes: Vec<f64> = results.iter()
        .filter_map(|r| r.map(|s| s))
        .collect();

    let success_percent = successes.len() * 100 / results.len();

    let formatted_percent = {
        match success_percent {
            100 => format!("{}{}", style(&success_percent).green(), style("%").green()),
            0 => format!("{}{}", style(&success_percent).red(), style("%").red()),
            _ => format!("{}{}", style(&success_percent).yellow(), style("%").yellow())
        }
    };

    println!("  Sent = {:.2}, Received = {:.2} ({})", results.len(), successes.len(), formatted_percent);

    if !successes.is_empty() {
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
        if last_char != &'.' {
            desc.push('.');
        }
    }

    // Collect Vec<char> -> String.
    desc.into_iter().collect()
}