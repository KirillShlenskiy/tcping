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
    let split: Vec<&str> = target.split(':').collect();
    
    assert!(split.len() != 0);

    let host = split[0];
    let mut port: Option<u16> = None;
    
    if split.len() > 1 {
        port = Some(split[1].parse::<u16>().unwrap());
    }

    if port.is_none() {
        println!("Port must be specified. See --help for more info.");
        return;
    }

    // DNS.
    let socket_parse_result = target.to_socket_addrs();

    if let Err(err) = socket_parse_result {
        println!("{} {}", style("DNS resolution failed.").red(), style(err).red());
        return;
    }

    let addr = socket_parse_result.unwrap().next().unwrap();

    println!("Connecting to {}", addr);

    // Connect.
    let res = TcpStream::connect_timeout(&addr, Duration::from_millis(5000));

    if let Err(err) = res {
        println!("{} {}", style("Can't connect.").red(), style(err).red());
        return;
    }

    println!("{}", style("Success!").green());
}