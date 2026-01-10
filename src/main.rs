use chrono::Local;
use clap::{Arg, Command};
use console::style;
use std::error::Error;
use std::io::{self, Error as IOError, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};
use tokio::time;

const DEFAULT_COUNT: u64 = 4;
const DEFAULT_COUNT_STR: &str = "4";
const DEFAULT_INTERVAL_MS: u64 = 1_000;
const DEFAULT_INTERVAL_MS_STR: &str = "1000";
const DEFAULT_TIMEOUT_MS: u64 = 4_000;

#[tokio::main]
async fn main() {
    let res = main_impl().await;

    if let Err(err) = res {
        println!("{} {}", style("Error:").red().bold(), err);
        std::process::exit(1);
    }
}

fn build_cli() -> Command {
    Command::new("tcping")
        .about(concat!(
            "TCP ping utility by Kirill Shlenskiy (2024) v",
            env!("CARGO_PKG_VERSION")
        ))
        .arg(
            Arg::new("target")
                .help("TCP ping target in \"host:port\" format (i.e. google.com:80)")
                .value_name("target")
                .required(true),
        )
        .arg(
            Arg::new("continuous")
                .short('t')
                .long("continuous")
                .action(clap::ArgAction::SetTrue)
                .help("Ping until stopped with Ctrl+C"),
        )
        .arg(
            Arg::new("count")
                .short('n')
                .long("count")
                .value_parser(clap::value_parser!(u64).range(1..))
                .default_value(DEFAULT_COUNT_STR)
                .help("Number of TCP requests (not counting warmup) to send"),
        )
        .arg(
            Arg::new("interval")
                .short('i')
                .long("interval")
                .value_parser(clap::value_parser!(u64).range(1..))
                .default_value(DEFAULT_INTERVAL_MS_STR)
                .help("Interval (in milliseconds) between requests"),
        )
        .arg_required_else_help(true)
}

async fn main_impl() -> Result<(), Box<dyn Error>> {
    let mut cmd = build_cli();

    if std::env::args().len() <= 1 {
        cmd.print_help().unwrap();
        return Ok(());
    }

    let matches = cmd.get_matches();
    let continuous = matches.get_flag("continuous");
    let count = *matches.get_one::<u64>("count").unwrap();
    let interval_ms = *matches.get_one::<u64>("interval").unwrap();
    let target = matches.get_one::<String>("target").unwrap();

    let addr = resolve_target(target)?;

    // Warmup.
    print_timed_ping(&addr, DEFAULT_TIMEOUT_MS, true, continuous);

    // Actual timed ping.
    let mut results = Vec::new();

    while continuous || (results.len() as u64) < count {
        time::sleep(Duration::from_millis(interval_ms)).await;
        results.push(print_timed_ping(
            &addr,
            DEFAULT_TIMEOUT_MS,
            false,
            continuous,
        ));
    }

    if !results.is_empty() {
        // Print stats (psping format):
        println!();
        print_stats(&results);
    }

    Ok(())
}

fn resolve_target(target: &str) -> Result<SocketAddr, IOError> {
    match target.to_socket_addrs() {
        Ok(mut addr_list) => addr_list.next().ok_or_else(|| {
            IOError::new(
                io::ErrorKind::InvalidInput,
                format!("No addresses resolved for '{}'.", target),
            )
        }),
        Err(err) => {
            let error_detail = fmt_err(&err);
            let message = if err.kind() == io::ErrorKind::InvalidInput {
                format!(
                    "Invalid target '{}'. Expected format: 'host:port' (i.e. 'google.com:80'). {}",
                    target, error_detail
                )
            } else {
                format!("Failed to resolve target '{}': {}", target, error_detail)
            };

            Err(IOError::new(err.kind(), message))
        }
    }
}

fn print_timed_ping(addr: &SocketAddr, timeout_ms: u64, warmup: bool, time: bool) -> Option<f64> {
    if warmup {
        if time {
            let now = Local::now().format("%H:%M:%S");
            print!("[{}] {} (warmup): ", &now, addr);
        } else {
            print!("> {} (warmup): ", addr);
        }

        io::stdout().flush().unwrap();
    } else if time {
        let now = Local::now().format("%H:%M:%S");
        print!("[{}] {}: ", &now, addr);
    } else {
        print!("> {}: ", addr);
    }

    match timed_ping(addr, timeout_ms) {
        Err(err) => {
            println!("{}", style(&err).cyan());
            None
        }
        Ok(latency_ms) => {
            println!("{:.2} ms", style(latency_ms).green().bold());
            Some(latency_ms)
        }
    }
}

fn timed_ping(addr: &SocketAddr, timeout_ms: u64) -> Result<f64, IOError> {
    let start = Instant::now();

    TcpStream::connect_timeout(addr, Duration::from_millis(timeout_ms))?;

    Ok(start.elapsed().as_secs_f64() * 1000.0)
}

fn print_stats(results: &[Option<f64>]) {
    let successes: Vec<f64> = results.iter().copied().flatten().collect();

    let success_percent = successes.len() * 100 / results.len();

    let formatted_percent = {
        match success_percent {
            100 => format!(
                "{}{}",
                style(&success_percent).green().bold(),
                style("%").green().bold()
            ),
            0 => format!(
                "{}{}",
                style(&success_percent).red().bold(),
                style("%").red().bold()
            ),
            _ => format!(
                "{}{}",
                style(&success_percent).yellow(),
                style("%").yellow()
            ),
        }
    };

    println!(
        "  Sent = {}, Received = {} ({})",
        results.len(),
        successes.len(),
        formatted_percent
    );

    if !successes.is_empty() {
        let min = successes
            .iter()
            .min_by(|&x, &y| x.partial_cmp(y).unwrap())
            .unwrap();

        let max = successes
            .iter()
            .max_by(|&x, &y| x.partial_cmp(y).unwrap())
            .unwrap();

        let avg = successes.iter().sum::<f64>() / successes.len() as f64;

        println!(
            "  Minimum = {:.2}ms, Maximum = {:.2}ms, Average = {:.2}ms",
            min, max, avg
        );
    }
}

fn fmt_err(err: &impl Error) -> String {
    let mut desc = Vec::new();
    {
        let desc_orig = err.to_string();
        let desc_chars = desc_orig.chars();

        for c in desc_chars {
            // Capitalise first letter.
            if desc.is_empty() && c.is_lowercase() {
                for u in c.to_uppercase() {
                    desc.push(u);
                }
            } else {
                desc.push(c);
            }
        }
    }

    // Ensure there is a full stop at the end.
    if let Some(last_char) = desc.last()
        && last_char != &'.'
    {
        desc.push('.');
    }

    // Collect Vec<char> -> String.
    desc.iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::net::TcpListener;

    #[test]
    fn test_timed_ping_success() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let result = timed_ping(&addr, DEFAULT_TIMEOUT_MS);
        assert!(result.is_ok());
    }

    #[test]
    fn test_timed_ping_timeout() {
        // Find an unused port.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);

        let result = timed_ping(&addr, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_fmt_err_formats_message() {
        let err = io::Error::new(io::ErrorKind::Other, "sample error");
        assert_eq!(fmt_err(&err), "Sample error.");
    }

    #[test]
    fn test_cli_defaults() {
        let matches = build_cli()
            .try_get_matches_from(["tcping", "localhost:80"])
            .unwrap();
        assert_eq!(*matches.get_one::<u64>("count").unwrap(), DEFAULT_COUNT);
        assert_eq!(
            *matches.get_one::<u64>("interval").unwrap(),
            DEFAULT_INTERVAL_MS
        );
    }

    #[test]
    fn test_cli_rejects_zero_count() {
        let result = build_cli().try_get_matches_from(["tcping", "-n", "0", "localhost:80"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_target_invalid() {
        let err = resolve_target("invalid-target").unwrap_err();
        let message = err.to_string();
        assert!(message.contains("Invalid target"));
        assert!(message.contains("host:port"));
    }

    #[test]
    fn test_print_stats() {
        let results = vec![Some(10.0), Some(20.0), Some(30.0), None];
        // This test will just execute the function to ensure it doesn't panic.
        // We can't easily assert the output without capturing stdout.
        print_stats(&results);
    }
}
