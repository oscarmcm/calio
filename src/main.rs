use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use almanac::Calendar;
use almanac::Date;
use almanac::Event;
use chrono::Duration;
use colored::Colorize;

fn print_day(date: Date) {
    println!("\n{}", date.format("%a %b %e %Y").green().bold())
}

fn print_event(event: &Event, ustart: bool, uend: bool, hide_desc: bool) {
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

    if !hide_desc && !event.description.is_empty() {
        let description = str::replace(&event.description, "\\n", &format!("\n{}", " ".repeat(16)));
        println!("{}{}", " ".repeat(16), description.cyan());
    }
}

fn print_events(events: impl Iterator<Item = Event>, hide_desc: bool) {
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
                            print_event(event, true, false, hide_desc);
                        } else {
                            print_event(event, true, true, hide_desc);
                        }
                    }
                }
            } else {
                day = event.start.clone();
                print_day(day);
            }
        }

        if event.end_date() > event.start + Duration::days(1) {
            print_event(&event, false, true, hide_desc);
            unfinish.push(event);
        } else {
            print_event(&event, false, false, hide_desc);
        }
    }

    while !unfinish.is_empty() {
        day = day + Duration::days(1);
        print_day(day);
        for (i, event) in unfinish.clone().iter().enumerate() {
            if event.end_date() <= day + Duration::days(1) {
                unfinish.remove(i);
                print_event(event, true, false, hide_desc);
            } else {
                print_event(event, true, true, hide_desc);
            }
        }
    }
    println!("");
}

fn main() {
    let help_text: String = format!(
        r"
{} 1.1.0
Tiny CLI tool that helps to visualize iCal file content in the terminal.

{}
    [STDIN] | calio [OPTIONS]
    calio [FILE_PATH | ICS_URL] [OPTIONS]

{}:
    {}    Keep the cli running and do not exit on stdout.
    {}     Don't show the event's description.
    {}          Display this message and exit.

{}:
    cat ~/invite.ics | calio
    calio ~/invite.ics --keep-alive
    calio ~/invite.ics --hide-desc
",
        "calio".green(),
        "USAGE:".yellow(),
        "OPTIONS:".yellow(),
        "--keep-alive".green(),
        "--help".green(),
        "--hide-desc".green(),
        "EXAMPLE".yellow()
    );

    let args: Vec<_> = env::args().collect();
    let is_stdin_empty: bool = atty::is(atty::Stream::Stdin);
    let mut keep_alive: bool = false;
    let hide_desc = args.iter().any(|arg| arg == "--hide-desc");

    if is_stdin_empty && args.len() < 2 {
        // no args no stdin
        println!("{}", "Not enough arguments suplied.".red());
        println!("{}", help_text);
        return;
    };

    if !is_stdin_empty {
        if args.len() >= 2 && args[1] != "--keep-alive".to_string() {
            // with stdin with file
            println!("{}", "Can't mix STDIN and FILE.".red());
            println!("{}", help_text);
            return;
        };
        if args.len() >= 1 {
            keep_alive = args[args.len() - 1] == "--keep-alive".to_string();
        };
        let stdin = std::io::stdin();
        let buf = BufReader::new(stdin);
        let calendars = Calendar::parse(buf).unwrap();
        print_events(calendars.iter(), hide_desc);
    };

    if is_stdin_empty {
        if args[1] == "--keep-alive".to_string() {
            println!("{}", "First argument must be a FILE.".red());
            println!("{}", help_text);
            return;
        };
        if ["-h".to_string(), "help".to_string(), "--help".to_string()].contains(&args[1]) {
            println!("{}", help_text);
            return;
        };
        if args.len() > 2 {
            keep_alive = args[2] == "--keep-alive".to_string();
        }

        let path = Path::new(&args[1]);
        let calendars = if path.exists() {
            let file = File::open(path).unwrap();
            let buf = BufReader::new(file);
            Calendar::parse(buf).unwrap()
        } else {
            // Print out a message while the calendar gets downloaded
            let s = format!(
                "{}{}",
                "Fetching calendar from ".dimmed(),
                &args[1].dimmed()
            );
            print!("{}", s);
            // Download the calendar from the given URL
            let url_text = reqwest::blocking::get(&args[1]).unwrap().text().unwrap();
            // Clear the printed message with \r
            // https://stackoverflow.com/a/35280799/14555505
            print!("\r{: <1$}", "", s.len());
            // Convert the url to a buffered reader
            let buf = BufReader::new(url_text.as_bytes());
            // Convert the buffered reader to a Calendar
            Calendar::parse(buf).unwrap()
        };
        print_events(calendars.iter(), hide_desc);
    };

    let running = Arc::new(AtomicBool::new(keep_alive));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");
    while running.load(Ordering::SeqCst) {}
}
