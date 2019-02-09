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
        .version("0.2")
        .about("TCP ping utility by Kirill Shlenskiy (2019)")
        .arg(Arg::from_usage("<target> 'TCP ping target in {host:port} format (i.e. google.com:80)'"))
        .arg(Arg::from_usage("-c, --count=[count] 'Number of requests (not counting warmup) to dispatch'"))
        .arg(Arg::from_usage("-t, --timeout=[timeout] 'Connection timeout in seconds; the default is 4'"))
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::DisableVersion)
        .get_matches();

    let count_match = matches.value_of("count");
    let mut count = 4; // Default.

    if let Some(count_str) = count_match {
        count = count_str.parse::<i32>().expect("Not a valid integer.");

        if count < 0 {
            println!("Invalid count.");
            return;
        }
    }

    let timeout_match = matches.value_of("timeout");
    let mut timeout_secs = 4; // Default.

    if let Some(timeout_str) = timeout_match {
        timeout_secs = timeout_str.parse::<i32>().expect("Not a valid integer.");

        if timeout_secs < 0 {
            println!("Invalid timeout.");
            return;
        }
    }

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
    timed_ping_disp(&addr, timeout_secs, true).ok();

    // Actual timed ping.
    let mut latencies = Vec::new();

    for _i in 0..count {
        thread::sleep(Duration::from_millis(500));

        if let Ok(latency_ms) = timed_ping_disp(&addr, timeout_secs, false) {
            latencies.push(latency_ms);
        }
    }

    // Print stats (psping format):
    println!();
    let received_percent = latencies.len() as i32 * 100 / count;

    let formatted_percent = {
        if received_percent == 100 {
            format!("{}{}", style(&received_percent).green(), style("%").green())
        }
        else {
            format!("{}{}", style(&received_percent).red(), style("%").red())
        }
    };

    println!("  Sent = {:.2}, Received = {:.2} ({})", count, latencies.len(), formatted_percent);

    if !latencies.is_empty() {
        println!(
            "  Minimum = {:.2}ms, Maximum = {:.2}ms, Average = {:.2}ms",
            aggregates::min(&latencies),
            aggregates::max(&latencies),
            aggregates::avg(&latencies)
        );
    }
}

fn timed_ping_disp(addr: &SocketAddr, timeout_secs: i32, warmup: bool) -> Result<f64, std::io::Error> {
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

fn timed_ping(addr: &SocketAddr, timeout_secs: i32) -> Result<f64, std::io::Error> {
    let start = Instant::now();

    if let Err(err) = TcpStream::connect_timeout(&addr, Duration::from_secs(timeout_secs as u64)) {
        return Err(err);
    }

    let finish = Instant::now();
    let diff = finish - start;
    let diff_ns = diff.subsec_nanos();
    let diff_ms = diff_ns as f64 / 1000000 as f64 + diff.as_secs() as f64 * 1000 as f64;

    Ok(diff_ms)
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