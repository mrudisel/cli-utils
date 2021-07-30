mod cli;
mod lib;

use finder::Finder;

fn main() {
    let args = match cli::parse_cli() {
        Ok(args) => args,
        Err(err) => panic!("{}", err),
    };

    if args.verbose {
        println!("{}", args);
    }
}
