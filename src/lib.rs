use clap::{App, Arg};
use std::error::Error;
use regex::{Regex, RegexBuilder};
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, BufRead};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    sources: Vec<String>,
    pattern: Option<Regex>,
    seed: Option<u64>,
}

#[derive(Debug)]
pub struct Fortune {
    source: String,
    text: String,
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

    let files = find_files(&config.sources)?;
    println!("{:#?}", files);
    Ok(())
}

fn parse_u64(val: &str) -> MyResult<u64> {
    val.parse().map_err(|_| format!("\"{}\" not a valid integer", val).into())
}

/*
    a function which takes in a list of strings representing paths
    It then finds all files in the given list of strings (recursively if the string represents a directory) and returns a sorted list
    of unique paths represented by MyResult<Vec<PathBuf>>
    Non-existent paths should return an error

 */

fn find_files(paths: &[String]) -> MyResult<Vec<PathBuf>> {

    let mut files = Vec::new();

    for path in paths {
        let path = Path::new(path);
        if path.exists() {
            if path.is_dir() {
                for entry in path.read_dir()? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_file() {
                        files.push(path);
                    }
                }
            } else if path.is_file() {
                files.push(path.to_path_buf());
            }
            
        }
        else {
            return Err(From::from(format!("{} does not exist", path.display())));
        }
       
    }

    let set:HashSet<_> = files.iter().cloned().collect();
    files = set.into_iter().collect();
    files.sort();
    // if vector is sorted we can also use files.dedup() which removes consecutive repated elements
    Ok(files)
}

fn read_fortunes(paths: &[PathBuf]) -> MyResult<Vec<Fortune>> {

    let mut fortunes = Vec::new();
    for path in paths {

        let mut file = BufReader::new(File::open(path)?);
        let mut buffer = Vec::new();
        let mut num_bytes = 0;
        loop {
            num_bytes = file.read_until(b'%', &mut buffer)?;
            if num_bytes == 0 {
                break;
            }
            fortunes.push(
                Fortune {
                    source: path.display().to_string(),
                    text: String::from_utf8(buffer.clone()).unwrap(),
                }
            );

        }
    }
    Ok(fortunes)
    
}
#[cfg(test)]
mod tests {
    use super::{
        find_files, parse_u64, //pick_fortune, read_fortunes, Fortune,
    };
    use std::path::PathBuf;

    #[test]
    fn test_parse_u64() {
        let res = parse_u64("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "\"a\" not a valid integer");

        let res = parse_u64("0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = parse_u64("4");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 4);
    }

    #[test]
    fn test_find_files() {
        // Verify that the function finds a file known to exist
        let res = find_files(&["./tests/inputs/jokes".to_string()]);
        assert!(res.is_ok());

        let files = res.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(
            files.get(0).unwrap().to_string_lossy(),
            "./tests/inputs/jokes"
        );

        // Fails to find a bad file
        let res = find_files(&["/path/does/not/exist".to_string()]);
        assert!(res.is_err());

        // Finds all the input files, excludes ".dat"
        let res = find_files(&["./tests/inputs".to_string()]);
        assert!(res.is_ok());

        // Check number and order of files
        let files = res.unwrap();
        assert_eq!(files.len(), 4);
        let first = files.get(0).unwrap().display().to_string();
        assert!(first.contains("ascii-art"));
        let last = files.last().unwrap().display().to_string();
        assert!(last.contains("quotes"));

        // Test for multiple sources, path must be unique and sorted
        let res = find_files(&[
            "./tests/inputs/jokes".to_string(),
            "./tests/inputs/ascii-art".to_string(),
            "./tests/inputs/jokes".to_string(),
        ]);
        assert!(res.is_ok());
        let files = res.unwrap();
        assert_eq!(files.len(), 2);
        if let Some(filename) = files.first().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "ascii-art".to_string())
        }
        if let Some(filename) = files.last().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "jokes".to_string())
        }
    }
}