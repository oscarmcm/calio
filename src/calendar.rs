use std::io::BufRead;
use std::fmt;
use ical::IcalParser;
use chrono::Duration;
use itertools::Itertools;

use date::Date;
use event::{Event, End};
use periodic::Periodic;
use errors::EventError;

pub struct Calendar {
    single: Vec<Event>,
    periodic: Vec<Periodic>,
}

impl Calendar {
    pub fn parse<B: BufRead>(buf: B) -> Result<Self, EventError> {
        let reader = IcalParser::new(buf);
        let mut single = Vec::new();
        let mut periodic = Vec::new();

        for line in reader {
            for ev in line?.events {
                let mut event = Event::new();
                let mut maybe_periodic = None;

                for property in ev.properties {
                    let value = property.value.unwrap_or("".to_string());
                    let mut time_zone = "".to_string();

                    let params = property.params.unwrap_or(vec![]);
                    for (param, value) in &params {
                        if param == "TZID" && value.len() > 0 {
                            time_zone = value[0].clone();
                        }
                    }

                    match property.name.as_ref() {
                        "SUMMARY" => event.summary = value,
                        "LOCATION" => event.location = value,
                        "DESCRIPTION" => event.description = value,
                        "STATUS" => event.status = value.parse()?,
                        "DTSTART" => event.start = Date::parse(&value, &time_zone)?,
                        "DTEND" => event.end = End::Date(Date::parse(&value, &time_zone)?),
                        "DURATION" => event.end = End::Duration(duration(&value)?),
                        "RRULE" => maybe_periodic = Some(rrule(&value, &params)?),
                        _ => (),
                    };
                }
                match maybe_periodic {
                    Some(mut p) => {
                        p.event = event;
                        periodic.push(p);
                    }
                    None => single.push(event),
                }
            }
        }

        single.sort();
        Ok(Calendar { single, periodic })
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = Event> + 'a {
        self.single.iter().map(Event::clone).merge(
            self.periodic
                .iter()
                .map(|p| p.iter())
                .kmerge(),
        )
    }
}

impl fmt::Display for Calendar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for event in &self.single {
            writeln!(f, "{}", event)?;
        }
        writeln!(f, "")?;
        for periodic in &self.periodic {
            writeln!(f, "{}", periodic)?;
        }
        Ok(())
    }
}

fn rrule(value: &str, params: &Vec<(String, Vec<String>)>) -> Result<Periodic, EventError> {
    let mut periodic = Periodic::new();

    for entry in value.split(";") {
        let p: Vec<&str> = entry.splitn(2, "=").collect();
        periodic.set_param(p[0], p[1])?;
    }

    for (param, values) in params {
        let mut value = "";
        if values.len() > 0 {
            value = &values[0];
        }
        periodic.set_param(param, value)?;
    }

    Ok(periodic)
}

fn duration(value: &str) -> Result<Duration, EventError> {
    let mut duration = Duration::seconds(0);
    let mut acc = "".to_string();
    for c in value.chars() {
        match c {
            '0'..='9' => acc.push(c),
            '-' => duration = -duration,
            'W' | 'H' | 'M' | 'S' | 'D' => {
                let count = acc.parse()?;
                acc = "".to_string();
                let d = match c {
                    'W' => Duration::weeks(count),
                    'H' => Duration::hours(count),
                    'M' => Duration::minutes(count),
                    'S' => Duration::seconds(count),
                    'D' => Duration::days(count),
                    _ => Duration::seconds(0),
                };
                duration = duration + d;
            }
            _ => (),
        }
    }
    Ok(duration)
}


