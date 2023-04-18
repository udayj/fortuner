use clap::{App, Arg};
use std::error::Error;
use regex::{Regex, RegexBuilder};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    sources: Vec<String>,
    pattern: Option<Regex>,
    seed: Option<u64>,
}

pub fn get_args() -> MyResult<Config> {

    let matches = App::new("fortuner")
                        .version("0.1.0")
                        .author("udayj")
                        .about("Rust fortune")
                        .arg(
                            Arg::with_name("sources")
                                .value_name("FILE")
                                .multiple(true)
                                .required(true)
                        )
                        .arg(
                            Arg::with_name("pattern")
                                .short("m")
                                .long("pattern")
                                .value_name("PATTERN")
                        )
                        .arg(
                            Arg::with_name("seed")
                                .short("s")
                                .long("seed")
                                .takes_value(true)
                        )
                        .arg(
                            Arg::with_name("insensitive")
                                .short("i")
                                .long("insensitive")
                                .takes_value(false)
                        )
                        .get_matches();

    let pattern = matches.value_of("pattern").map(
        |val| {
            RegexBuilder::new(val)
                .case_insensitive(matches.is_present("insensitive"))
                .build()
                .map_err(|_| format!("Invalid pattern \"{}\"",val))       
        }
    ).transpose()?;

    Ok(
        Config {
            sources: matches.values_of_lossy("sources").unwrap(),
            seed: matches.value_of("seed").map(parse_u64).transpose()?,
            pattern,

        }
    )
    
}

pub fn run(config: Config) -> MyResult<()> {

    println!("{:#?}", config);
    Ok(())
}

fn parse_u64(val: &str) -> MyResult<u64> {

    val.parse().map_err(|_| format!("\"{}\" not a valid integer", val).into())
}