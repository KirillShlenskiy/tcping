use std::error::*;
use std::net::{ToSocketAddrs,TcpStream};
use std::time::Duration;

use clap::{App,AppSettings,Arg};
use console::style;

fn main() {
    let matches = App::new("tcping")
        .version("0.1")
        .about("TCP ping utility by Kirill Shlenskiy (2019)")
        .arg(Arg::from_usage("<target> 'TCP ping target in {host:port} format (i.e. google.com:80)'"))
        .arg(Arg::from_usage("-t, --timeout 'Connection timeout in milliseconds'"))
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::DisableVersion)
        .get_matches();

    let target = matches.value_of("target").unwrap();
    let socket_addr_result = target.to_socket_addrs();

    if let Err(err) = socket_addr_result {
        // Format error.
        println!("{}", style(fmt_err(&err)).red());
        return;
    }

    let addr = socket_addr_result.unwrap().next().unwrap();
    println!("Connecting to {}", addr);

    // Connect.
    let res = TcpStream::connect_timeout(&addr, Duration::from_millis(5000));

    if let Err(err) = res {
        println!("{} {}", style("Can't connect.").red(), style(err).red());
        return;
    }

    println!("{}", style("Success!").green());
}

fn fmt_err(err: &Error) -> String {
    let mut desc = Vec::new();
    {
        let mut desc_chars = err.description().chars();

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