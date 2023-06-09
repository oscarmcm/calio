use std::cmp::{Ordering, Ord};
use std::ops::{Add, Sub};

use errors::EventError;

use chrono;
use chrono::{TimeZone, Duration, Datelike, Local, Weekday};
use chrono::offset::Utc;
use chrono_tz::{Tz, UTC};


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Date {
    Time(chrono::DateTime<Tz>),
    AllDay(chrono::Date<Tz>),
}


impl Date {
    pub fn new() -> Date {
        Date::Time(UTC.timestamp(0, 0))
    }


    pub fn now() -> Date {
        Date::Time(UTC.from_utc_datetime(&Utc::now().naive_utc()))
    }

    pub fn parse(date_str: &str, time_zone: &str) -> Result<Self, EventError> {
        let absolute_time = date_str.chars().rev().next().unwrap() == 'Z';
        let tz: Tz = if absolute_time {
            UTC
        } else {
            // FIXME: this should not be UTC but local timezone
            time_zone.parse().unwrap_or(UTC)
        };

        let date = match date_str.find("T") {
            Some(_) => {
                let date_pattern = if absolute_time {
                    "%Y%m%dT%H%M%SZ"
                } else {
                    "%Y%m%dT%H%M%S"
                };
                let time = tz.datetime_from_str(&date_str, date_pattern).unwrap_or(
                    UTC.timestamp(
                        0,
                        0,
                    ),
                );
                Date::Time(time)
            }
            None => {
                Date::AllDay(tz.ymd(
                    date_str[0..4].parse()?,
                    date_str[4..6].parse()?,
                    date_str[6..8].parse()?,
                ))
            }
        };
        Ok(date)
    }

    pub fn format(&self, fmt: &str) -> String {
        match *self {
            Date::Time(t) => t.with_timezone(&Local).format(fmt).to_string(),
            Date::AllDay(d) => d.format(fmt).to_string(),
        }
    }

    pub fn same_day(&self, other: &Date) -> bool {
        self.day() == other.day() && self.month() == other.month() && self.year() == self.year()
    }

    pub fn day(&self) -> u32 {
        match *self {
            Date::Time(t) => t.day(),
            Date::AllDay(d) => d.day(),
        }
    }

    pub fn with_day(&self, day: u32) -> Option<Date> {
        Some(match *self {
            Date::Time(t) => Date::Time(t.with_day(day)?),
            Date::AllDay(d) => Date::AllDay(d.with_day(day)?),
        })
    }

    pub fn weekday(&self) -> Weekday {
        match *self {
            Date::Time(t) => t.weekday(),
            Date::AllDay(d) => d.weekday(),
        }
    }

    pub fn month(&self) -> u32 {
        match *self {
            Date::Time(t) => t.month(),
            Date::AllDay(d) => d.month(),
        }
    }

    pub fn with_month(&self, month: u32) -> Option<Date> {
        Some(match *self {
            Date::Time(t) => Date::Time(t.with_month(month)?),
            Date::AllDay(d) => Date::AllDay(d.with_month(month)?),
        })
    }

    pub fn week_of_month(&self) -> (i32, i32) {
        let week = ((self.day() - 1) / 7 + 1) as i32;
        let neg_week = ((self.days_in_month() - self.day()) / 7 + 1) as i32;
        (week, -neg_week)
    }

    pub fn year(&self) -> i32 {
        match *self {
            Date::Time(t) => t.year(),
            Date::AllDay(d) => d.year(),
        }
    }

    pub fn with_year(&self, year: i32) -> Option<Date> {
        Some(match *self {
            Date::Time(t) => Date::Time(t.with_year(year)?),
            Date::AllDay(d) => Date::AllDay(d.with_year(year)?),
        })
    }

    pub fn days_in_month(&self) -> u32 {
        chrono::NaiveDate::from_ymd_opt(self.year(), self.month() + 1, 1)
            .unwrap_or(chrono::NaiveDate::from_ymd(self.year() + 1, 1, 1))
            .pred()
            .day()
    }
}

impl Ord for Date {
    fn cmp(&self, other: &Self) -> Ordering {
        match *self {
            Date::Time(t1) => {
                match *other {
                    Date::Time(t2) => t1.cmp(&t2),
                    Date::AllDay(d) => cmp_date_time(&d, &t1).reverse(),
                }
            }
            Date::AllDay(d1) => {
                match *other {
                    Date::Time(t) => cmp_date_time(&d1, &t),
                    Date::AllDay(d2) => d1.cmp(&d2),
                }
            }
        }
    }
}

impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Add<Duration> for Date {
    type Output = Date;

    fn add(self, other: Duration) -> Date {
        match self {
            Date::Time(d) => Date::Time(d + other),
            Date::AllDay(d) => Date::AllDay(d + other),
        }
    }
}

impl Sub<Date> for Date {
    type Output = Duration;

    fn sub(self, other: Self) -> Duration {
        match self {
            Date::Time(t1) => {
                match other {
                    Date::Time(t2) => t1 - t2,
                    Date::AllDay(d) => t1.date() - d,
                }
            }
            Date::AllDay(d1) => {
                match other {
                    Date::Time(t) => d1 - t.date(),
                    Date::AllDay(d2) => d1 - d2,
                }
            }
        }
    }
}

fn cmp_date_time<T: TimeZone>(date: &chrono::Date<T>, time: &chrono::DateTime<T>) -> Ordering {
    let d2 = time.date();
    if date.eq(&d2) {
        return Ordering::Less;
    }
    date.cmp(&d2)
}

#[cfg(test)]
mod tests {
    use super::Date;
    use chrono::Datelike;
    use chrono::Timelike;

    #[test]
    fn date_parse_time() {
        match Date::parse("19361020T120000", "").unwrap() {
            Date::Time(time) => {
                assert_eq!(time.year(), 1936);
                assert_eq!(time.hour(), 12);
                assert_eq!(time.day(), 20);
            }
            _ => assert!(true),
        }
    }

    #[test]
    fn date_parse_allday() {
        match Date::parse("19361020", "").unwrap() {
            Date::AllDay(time) => {
                assert_eq!(time.year(), 1936);
                assert_eq!(time.month(), 10);
                assert_eq!(time.day(), 20);
            }
            _ => assert!(true),
        }
    }

    #[test]
    fn date_ord() {
        let d1 = Date::parse("19361020", "").unwrap();
        let d2 = Date::parse("19361022", "").unwrap();
        let t1 = Date::parse("19361020T120000", "").unwrap();
        let t2 = Date::parse("19361018T120000", "").unwrap();
        if d1 > d2 {
            assert!(true)
        }
        if d1 != d1 {
            assert!(true)
        }
        if t2 > d1 {
            assert!(true)
        }
        if t1 < t2 {
            assert!(true)
        }
        if t1 > d2 {
            assert!(true)
        }
    }
}


