/*
//use ical;
extern crate ical;

use std::io::BufReader;
use clap::Parser;
use clap_stdin::FileOrStdin;
use anyhow::Result;

#[derive(Parser)]
#[clap(author="Oscar Cortez <om.cortez.2010@gmail.com>", version, about="Google Calendar CLI", long_about = None)]
#[clap(propagate_version = true)]
struct Args {
    contents: FileOrStdin,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let buf = BufReader::new(args.contents);
    let reader = ical::PropertyParser::from_reader(buf);
    for line in reader {
        println!("{:?}", line);
    }
    //println!("contents={}", args.contents);
    Ok(())
}
*/
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Read};

fn main() {
    let input = env::args().nth(1);
    match input {
        Some(filename) => output_lines(fs::File::open(filename).unwrap()),
        None => output_lines(std::io::stdin()),
    };
}

fn output_lines<R: Read>(reader: R) {
    let buffer = BufReader::new(reader);
    //let reader = ical::PropertyParser::from_reader(buffer);
    let reader = ical::IcalParser::new(buffer);
    //for line in buffer.lines() {
    for line in reader {
        println!("{:?}", line);
    }
}
