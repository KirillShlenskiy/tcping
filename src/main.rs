use std::error::*;
use std::net::{SocketAddr,ToSocketAddrs,TcpStream};
use std::time::Duration;

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
        println!("{}", style(fmt_err(&err)).red());
        return;
    }

    let addr = socket_addr_result.unwrap().next().unwrap();

    // Warmup.
    print!("Connecting to {} (warmup): ", addr);
    ping(&addr);

    // Actual ping.
    for i in 0..count {
        print!("Connecting to {}: ", addr);
        ping(&addr);
    }
}

fn ping(addr: &SocketAddr) {
    let res = TcpStream::connect_timeout(&addr, Duration::from_millis(5000));

    if let Err(err) = res {
        println!(); // Complete current line.
        println!("{} {}", style("Can't connect.").red(), style(err).red());
        return;
    }

    println!("{}", style("success!").green());
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