use std::path::PathBuf;
use clap::{Arg, App, ArgMatches};

pub struct ForEachArgs {
    inputs: Vec<PathBuf>,
    cmd: &'static str,
    args: Vec<&'static str>,
}




pub fn parse_cli_args() -> Result<ForEachArgs, &'static str> {
    let threads_arg = Arg::with_name("max-threads")
    .help("Maximum number of threads to use")

    Err("err")
}
