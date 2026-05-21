use chrono::Local;
use clap::{Arg, Command};
use console::style;
use std::error::Error;
use std::io::{self, Error as IoError, Write};
use std::net::{SocketAddr, ToSocketAddrs};
use std::time::{Duration, Instant};
use tokio::{net::TcpStream, signal, time};

const DEFAULT_TIMEOUT_MS: u64 = 4_000;

#[tokio::main]
async fn main() {
    let res = main_impl().await;

    if let Err(err) = res {
        eprintln!("{} {}", style("Error:").red().bold(), err);
        std::process::exit(1);
    }
}

fn build_cli() -> Command {
    Command::new("tcping")
        .about(concat!(
            "TCP ping utility by Kirill Shlenskiy (2026) v",
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
                .default_value("4")
                .help("Number of TCP requests (not counting warmup) to send"),
        )
        .arg(
            Arg::new("interval")
                .short('i')
                .long("interval")
                .value_parser(clap::value_parser!(u64).range(1..))
                .default_value("1000")
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
    print_timed_ping(&addr, DEFAULT_TIMEOUT_MS, true, continuous).await;

    // Actual timed ping.
    let mut stats = PingStats::new();

    if continuous {
        let ctrl_c = signal::ctrl_c();
        tokio::pin!(ctrl_c);
        'ping_loop: loop {
            tokio::select! {
                _ = &mut ctrl_c => {
                    break 'ping_loop;
                }
                () = time::sleep(Duration::from_millis(interval_ms)) => {
                    let result = tokio::select! {
                        _ = &mut ctrl_c => {
                            break 'ping_loop;
                        }
                        result = print_timed_ping(
                            &addr,
                            DEFAULT_TIMEOUT_MS,
                            false,
                            continuous,
                        ) => result,
                    };
                    stats.record(result);
                }
            }
        }
    } else {
        for _ in 0..count {
            time::sleep(Duration::from_millis(interval_ms)).await;
            let result = print_timed_ping(&addr, DEFAULT_TIMEOUT_MS, false, continuous).await;
            stats.record(result);
        }
    }

    if stats.sent > 0 {
        // Print stats (psping format):
        println!();
        print_stats(&stats);
    }

    Ok(())
}

fn resolve_target(target: &str) -> Result<SocketAddr, IoError> {
    match target.to_socket_addrs() {
        Ok(mut addr_list) => addr_list.next().ok_or_else(|| {
            IoError::new(
                io::ErrorKind::InvalidInput,
                format!("No addresses resolved for '{target}'."),
            )
        }),
        Err(err) => {
            let error_detail = fmt_err(&err);
            let message = if err.kind() == io::ErrorKind::InvalidInput {
                format!(
                    "Invalid target '{target}'. Expected format: 'host:port' (i.e. 'google.com:80'). {error_detail}"
                )
            } else {
                format!("Failed to resolve target '{target}': {error_detail}")
            };

            Err(IoError::new(err.kind(), message))
        }
    }
}

async fn print_timed_ping(
    addr: &SocketAddr,
    timeout_ms: u64,
    warmup: bool,
    show_timestamp: bool,
) -> Option<f64> {
    if warmup {
        if show_timestamp {
            let now = Local::now().format("%H:%M:%S");
            print!("[{}] {} (warmup): ", &now, addr);
        } else {
            print!("> {addr} (warmup): ");
        }

        io::stdout().flush().unwrap();
    } else if show_timestamp {
        let now = Local::now().format("%H:%M:%S");
        print!("[{}] {}: ", &now, addr);
    } else {
        print!("> {addr}: ");
    }

    match timed_ping(addr, timeout_ms).await {
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

async fn timed_ping(addr: &SocketAddr, timeout_ms: u64) -> Result<f64, IoError> {
    let start = Instant::now();

    let connect = TcpStream::connect(addr);
    match time::timeout(Duration::from_millis(timeout_ms), connect).await {
        Ok(Ok(_stream)) => Ok(start.elapsed().as_secs_f64() * 1000.0),
        Ok(Err(err)) => Err(err),
        Err(_) => Err(IoError::new(
            io::ErrorKind::TimedOut,
            "connection timed out",
        )),
    }
}

struct PingStats {
    sent: usize,
    received: usize,
    min: Option<f64>,
    max: Option<f64>,
    sum: f64,
}

impl PingStats {
    fn new() -> Self {
        Self {
            sent: 0,
            received: 0,
            min: None,
            max: None,
            sum: 0.0,
        }
    }

    fn record(&mut self, result: Option<f64>) {
        self.sent += 1;
        if let Some(latency) = result {
            self.received += 1;
            self.sum += latency;
            self.min = Some(self.min.map_or(latency, |min| min.min(latency)));
            self.max = Some(self.max.map_or(latency, |max| max.max(latency)));
        }
    }

    fn avg(&self) -> Option<f64> {
        if self.received == 0 {
            None
        } else {
            Some(self.sum / self.received as f64)
        }
    }
}

fn print_stats(stats: &PingStats) {
    let success_percent = stats.received * 100 / stats.sent;

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
        stats.sent, stats.received, formatted_percent
    );

    if let (Some(min), Some(max), Some(avg)) = (stats.min, stats.max, stats.avg()) {
        println!("  Minimum = {min:.2}ms, Maximum = {max:.2}ms, Average = {avg:.2}ms");
    }
}

fn fmt_err(err: &impl Error) -> String {
    let message = err.to_string();
    let mut chars = message.chars();
    let mut desc = match chars.next() {
        Some(first) => {
            let mut formatted = first.to_uppercase().collect::<String>();
            formatted.extend(chars);
            formatted
        }
        None => String::new(),
    };

    if !desc.ends_with('.') {
        desc.push('.');
    }

    desc
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::net::TcpListener;

    #[tokio::test]
    async fn test_timed_ping_success() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let result = timed_ping(&addr, DEFAULT_TIMEOUT_MS).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_timed_ping_connection_failure() {
        // Find an unused port.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);

        let result = timed_ping(&addr, 1).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_fmt_err_formats_message() {
        let err = io::Error::other("sample error");
        assert_eq!(fmt_err(&err), "Sample error.");
    }

    #[test]
    fn test_cli_defaults() {
        let matches = build_cli()
            .try_get_matches_from(["tcping", "localhost:80"])
            .unwrap();
        assert_eq!(*matches.get_one::<u64>("count").unwrap(), 4);
        assert_eq!(*matches.get_one::<u64>("interval").unwrap(), 1_000);
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
        let mut stats = PingStats::new();
        for result in [Some(10.0), Some(20.0), Some(30.0), None] {
            stats.record(result);
        }
        // This test will just execute the function to ensure it doesn't panic.
        // We can't easily assert the output without capturing stdout.
        print_stats(&stats);
    }
}
