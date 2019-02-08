use clap::{App,AppSettings,Arg};
use console::style;

fn main() {
    let matches = App::new("tcping")
        .version("0.1")
        .about("TCP ping utility by Kirill Shlenskiy (2019)")
        .arg(Arg::from_usage("<target> 'TCP ping target: {host:port} format (i.e. google.com:80).'"))
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::DisableVersion)
        .get_matches();

    let target = matches.value_of("target").unwrap();

    println!("{}", target);
}