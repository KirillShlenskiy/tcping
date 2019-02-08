use std::error::Error;
use std::io;
use std::io::prelude::*;
use std::net::{SocketAddr,ToSocketAddrs,TcpStream};
use std::time::{Duration,Instant};
use std::thread;

use clap::{App,AppSettings,Arg};
use console::style;

fn main() {
    let matches = App::new("tcping")
        .version("0.1")
        .about("TCP ping utility by Kirill Shlenskiy (2019)")
        .arg(Arg::from_usage("<target> 'TCP ping target in {host:port} format (i.e. google.com:80)'"))
        .arg(Arg::from_usage("-c, --count 'Number of requests (not counting warmup) to issue"))
        .arg(Arg::from_usage("-t, --timeout 'Connection timeout in milliseconds'"))
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

    let target = matches.value_of("target").unwrap();
    let socket_addr_result = target.to_socket_addrs();

    if let Err(err) = socket_addr_result {
        // Format error.
        println!("{}", style(&err).red());
        return;
    }

    let addr = socket_addr_result.unwrap().next().unwrap();

    // Warmup.
    print!("> {} (warmup): ", addr);
    io::stdout().flush().unwrap();
    ping(&addr).unwrap_or_default();

    // Actual timed ping.
    let mut latencies = Vec::new();

    for _i in 0..count {
        thread::sleep(Duration::from_millis(500));
        print!("> {}: ", addr);

        if let Some(latency) = ping(&addr).ok() {
            latencies.push(latency);
        }
    }

    // Print stats (psping format):
    println!();
    println!("  Sent = {:.2}, Received = {:.2}", count, latencies.len());

    if !latencies.is_empty() {
        println!("  Minimum = {:.2}ms, Maximum = {:.2}ms, Average = {:.2}ms", min(&latencies), max(&latencies), avg(&latencies));
    }
}

fn ping(addr: &SocketAddr) -> Result<f64, std::io::Error> {
    let start = Instant::now();
    let res = TcpStream::connect_timeout(&addr, Duration::from_millis(5000));

    if let Err(err) = res {
        println!("{}", style(&err).red());
        Err(err)
    }
    else {
        let finish = Instant::now();
        let diff = finish - start;
        let diff_ns = diff.subsec_nanos();
        let diff_ms = diff_ns as f64 / 1000000 as f64 + diff.as_secs() as f64 * 1000 as f64;

        println!("{:.2} ms", diff_ms);
        Ok(diff_ms)
    }
}

fn min(numbers: &[f64]) -> &f64 {
    let mut i = numbers.iter();
    let mut m = i.next().unwrap();

    while let Some(n) = i.next() {
        if n < m {
            m = n;
        }
    }

    m
}

fn max(numbers: &[f64]) -> &f64 {
    let mut i = numbers.iter();
    let mut m = i.next().unwrap();

    while let Some(n) = i.next() {
        if n > m {
            m = n;
        }
    }

    m
}

fn avg(numbers: &[f64]) -> f64 {
    numbers.iter().sum::<f64>() as f64 / numbers.len() as f64
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