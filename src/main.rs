extern crate chrono;
extern crate clap;
extern crate console;

use std::cmp::PartialOrd;
use std::error::Error;
use std::io::{self, Error as IOError, Write};
use std::net::{SocketAddr, ToSocketAddrs, TcpStream};
use std::str::FromStr;
use std::time::{Duration, Instant};
use tokio::time;

use crate::chrono::Local;
use crate::clap::{App, AppSettings, Arg, ArgMatches};
use crate::console::{style};

const TIMEOUT_SECS: u64 = 4;

#[tokio::main]
async fn main() {
    let res = main_impl().await;

    if let Err(err) = res {
        println!("{} {}", style("Error:").red().bold(), err);
        std::process::exit(1);
    }
}

async fn main_impl() -> Result<(), Box<dyn Error>> {
    let matches = App::new("tcping")
        .version("0.9.8")
        .about("TCP ping utility by Kirill Shlenskiy (2024)")
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
                if err.to_string() == "invalid socket address" {
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
    print_timed_ping(&addr, TIMEOUT_SECS, true, continuous);

    // Actual timed ping.
    let mut results = Vec::new();

    while continuous || (results.len() as u64) < count {
        time::sleep(Duration::from_millis(interval_ms)).await;
        results.push(print_timed_ping(&addr, TIMEOUT_SECS, false, continuous));
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

fn print_timed_ping(addr: &SocketAddr, timeout_secs: u64, warmup: bool, time: bool) -> Option<f64> {
    if warmup {
        if time {
            let now = Local::now().format("%H:%M:%S");
            print!("[{}] {} (warmup): ", &now, addr);
        }
        else {
            print!("> {} (warmup): ", addr);
        }
        
        io::stdout().flush().unwrap();
    }
    else {
        if time {
            let now = Local::now().format("%H:%M:%S");
            print!("[{}] {}: ", &now, addr);
        }
        else {
            print!("> {}: ", addr);
        }
    }

    match timed_ping(&addr, timeout_secs) {
        Err(err) => {
            println!("{}", style(&err).cyan());
            None
        },
        Ok(latency_ms) => {
            println!("{:.2} ms", style(latency_ms).green().bold());
            Some(latency_ms)
        }
    }
}

fn timed_ping(addr: &SocketAddr, timeout_secs: u64) -> Result<f64, IOError> {
    let start = Instant::now();

    if let Err(err) = TcpStream::connect_timeout(&addr, Duration::from_secs(timeout_secs)) {
        return Err(err);
    }

    let finish = Instant::now();
    let diff = finish - start;
    let diff_ns = diff.subsec_nanos();
    let diff_ms = diff_ns as f64 / 1_000_000_f64 + diff.as_secs() as f64 * 1_000_f64;

    Ok(diff_ms)
}

fn print_stats(results: &[Option<f64>]) {
    let successes: Vec<f64> = results.iter()
        .filter_map(|&r| r)
        .collect();

    let success_percent = successes.len() * 100 / results.len();

    let formatted_percent = {
        match success_percent {
            100 => format!("{}{}", style(&success_percent).green().bold(), style("%").green().bold()),
            0 => format!("{}{}", style(&success_percent).red().bold(), style("%").red().bold()),
            _ => format!("{}{}", style(&success_percent).yellow(), style("%").yellow())
        }
    };

    println!("  Sent = {:.2}, Received = {:.2} ({})", results.len(), successes.len(), formatted_percent);

    if !successes.is_empty() {
        let min = successes.iter()
            .min_by(|&x, &y| x.partial_cmp(y).unwrap())
            .unwrap();

        let max = successes.iter()
            .max_by(|&x, &y| x.partial_cmp(y).unwrap())
            .unwrap();

        let avg = successes.iter()
            .sum::<f64>() / successes.len() as f64;

        println!("  Minimum = {:.2}ms, Maximum = {:.2}ms, Average = {:.2}ms", min, max, avg);
    }
}

fn fmt_err(err: &impl Error) -> String
{
    let mut desc = Vec::new();
    {
        let desc_orig = err.to_string();
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
    desc.iter().collect()
}