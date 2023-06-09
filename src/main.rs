use std::env;
use std::fs::File;
use std::io::{BufReader};

use chrono::Duration;
use colored::Colorize;
use almanac::Calendar;
use almanac::Date;
use almanac::Event;

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


fn main() {
    match env::args().nth(1) {
        Some(filename) => {
            if ["-h".to_string(), "help".to_string(), "--help".to_string()]
                .contains(&filename.to_owned())
            {
                println!("Print help");
                return;
            } else {
                let file = File::open(filename).unwrap();
                let buf = BufReader::new(file);
                let calendars = Calendar::parse(buf).unwrap();
                print_events(calendars.iter())
            }
        }
        None => {
            if atty::is(atty::Stream::Stdin) {
                println!("Missing Args");
                return;
            } else {
                let stdin = std::io::stdin();
                let buf = BufReader::new(stdin);
                let calendars = Calendar::parse(buf).unwrap();
                print_events(calendars.iter())
            }
        }
    };
}
