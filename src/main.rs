#[macro_use]
extern crate clap;

use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::iter::Iterator;
use std::str;

use clap::{App, Arg};
use itertools::Itertools;

mod shell_runner;

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

    let mut strs: Vec<(&str, &str)> = strings
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
        let values: Vec<String> = v.into_iter().map(|t| t.1.to_owned()).collect_vec();
        (k, match map_expression {
            None => values,
            Some(expr) => shell_runner::execute(values, expr).unwrap().0 // FIXME unwrap
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
            let values: Vec<String> = map_result.into_iter().flat_map(|x| x.1).collect_vec();
            for value in shell_runner::execute(values, expr).unwrap().0 /* FIXME unwrap */ {
                println!("{}", value);
            }
        }
    }
}