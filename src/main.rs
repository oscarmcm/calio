use itertools::Itertools;
use colored::*;
use atty;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Read, stdin};
use std::fs::File;

use Calio::Calendar;
use Calio::Date;
use Calio::Event;

fn ics_file_calendar(file_path: &str) -> Calendar {
    let file = File::open(file_path).unwrap();
    let buf = BufReader::new(file);
    Calendar::parse(buf).unwrap()
}

fn ics_std_calendar(input: &stdin) -> Calendar {
    let buf = BufReader::new(input);
    Calendar::parse(buf).unwrap()
}

fn print_day(date: Date) {
    println!("\n{}", date.format("%a %b %e %Y").green().bold())
}

fn print_event(event: &Event, ustart: bool, uend: bool) {
    let start = if ustart {
        "-----".to_string()
    } else {
        match event.start {
            Date::Time(_) => event.start.format("%R"),
            Date::AllDay(_) => "-----".to_string(),
        }
    };
    let end = if uend {
        "-----".to_string()
    } else {
        match event.end_date() {
            Date::Time(_) => event.end_date().format("%R"),
            Date::AllDay(_) => "-----".to_string(),
        }
    };

    println!(
        "    {}-{} {} {}",
        start.yellow(),
        end.yellow(),
        event.summary,
        event.location.purple()
    );

    if !event.description.is_empty() {
        let description = str::replace(&event.description, "\\n", &format!("\n{}", " ".repeat(16)));
        println!("{}{}", " ".repeat(16), description.cyan());
    }
}

fn print_events(events: impl Iterator<Item = Event>) {
    let mut day = Date::new();
    let mut unfinish: Vec<Event> = vec![];

    for event in events {
        if !day.same_day(&event.start) {
            if !unfinish.is_empty() {
                while !day.same_day(&event.start) {
                    day = day + Duration::days(1);
                    print_day(day);
                    for (i, event) in unfinish.clone().iter().enumerate() {
                        if event.end_date() <= day + Duration::days(1) {
                            unfinish.remove(i);
                            print_event(event, true, false);
                        } else {
                            print_event(event, true, true);
                        }
                    }
                }
            } else {
                day = event.start.clone();
                print_day(day);
            }
        }

        if event.end_date() > event.start + Duration::days(1) {
            print_event(&event, false, true);
            unfinish.push(event);
        } else {
            print_event(&event, false, false);
        }
    }

    while !unfinish.is_empty() {
        day = day + Duration::days(1);
        print_day(day);
        for (i, event) in unfinish.clone().iter().enumerate() {
            if event.end_date() <= day + Duration::days(1) {
                unfinish.remove(i);
                print_event(event, true, false);
            } else {
                print_event(event, true, true);
            }
        }
    }
    println!("");
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


fn main() {
    let input = env::args().nth(1);
    match input {
        Some(filename) => {
            if ["-h", "help", "--help"].contains(&filename) {
                println!("Print help")
            } else {
                let mut calendars = ics_file_calendar(&filename);
            }
            //output_lines(fs::File::open(filename).unwrap()),
        },
        None => {
            if atty::is(atty::Stream::Stdin) {
                println!("Missing Args")
                return;
            } else {
                let buf = stdin();
                let mut calendars = ics_std_calendar(buf);
            }
            //output_lines(std::io::stdin()),
        }
    };
    let events = calendars
        .iter()
        .map(|c| c.iter())
        .kmerge()
        .skip_while(|e| e.end_date() < first)
        .take_while(|e| e.start <= last);
    print_events(events)

}

