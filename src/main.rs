#[macro_use]
extern crate clap;

use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::io::Write;
use std::iter::Iterator;
use std::process::Command;
use std::process::Stdio;
use std::str;

use clap::{App, Arg};
use itertools::Itertools;

fn main() {
    let matches = App::new("gb")
        .version("1.0")
        .author("Guillaume SR <tcherno@gmail.com>")
        .about("group by")
        .arg(
            Arg::with_name("column")
                .short("c")
                .long("column")
                .takes_value(true)
                .help("Sets the column for grouping")
        )
        .arg(
            Arg::with_name("map")
                .short("m")
                .long("map")
                .takes_value(true)
                .help("Shell command to run against each group")
        )
        .arg(
            Arg::with_name("reduce")
                .short("r")
                .long("reduce")
                .takes_value(true)
                .help("Shell command to run against the result")
        )
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .index(1)
        )
        .get_matches();

    let reader: Box<BufRead> = match matches.value_of("INPUT") {
        None => Box::new(BufReader::new(io::stdin())),
        Some(file) => Box::new(BufReader::new(File::open(file).expect("file not found"))),
    };

    let strings = reader.lines()
        .map(|l| l.unwrap())
        .filter(|l| { !l.is_empty() })
        .collect_vec();

    let column = value_t!(matches, "column", usize);

    let mut strs = strings
        .iter()
        .map(|s| &s[..])
        .map(|s| {
            let key = match column {
                Err(_) => s,
                Ok(column_index) => {
                    let vec = s.split_whitespace().collect_vec();
                    if vec.len() > column_index { vec[column_index] } else { "" }
                }
            };
            (key, s)
        })
        .collect_vec();

    strs.sort_unstable();

    let group = strs.into_iter().group_by(|tuple| tuple.0);

    let map_expression = matches.value_of("map");

    let map_result = group.into_iter().map(|(k, v)| {
        let values: Vec<&str> = v.into_iter().map(|t| t.1).collect_vec();
        (k, match map_expression {
            None => values.into_iter().map(str::to_owned).collect_vec(),
            Some(expr) => run(values, expr)
        })
    }).collect_vec();

    let reduce_expression = matches.value_of("reduce");

    match reduce_expression {
        None => for (k, values) in map_result {
            for value in values {
                println!("{} - {}", k, value);
            }
        }
        Some(expr) => {
            let values: Vec<&str> = map_result.iter().flat_map(|x| x.1.iter().map(|s| &s[..]).collect_vec()).collect_vec();
            for value in run(values, expr) {
                println!("{}", value);
            }
        }
    }
}

fn run(values: Vec<&str>, expr: &str) -> Vec<String> {
    let mut cmd = Command::new("sh")
        .arg("-c")
        .arg(expr)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        // handle stderr
        .spawn()
        .unwrap();
    cmd.stdin
        .as_mut()
        .unwrap()
        .write_all((values.join("\n") + "\n").as_bytes()).unwrap();
    let stdout = cmd.wait_with_output().unwrap().stdout;
    let output = String::from_utf8(stdout).unwrap();
    output.split("\n").map(str::to_owned)
        .filter(|l| { !l.is_empty() }).collect_vec()
}
