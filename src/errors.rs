use std::io;
use std::num::ParseIntError;
use ical::parser::ParserError;

#[derive(Debug)]
pub enum EventError {
    IcalError(ParserError),
    IntError(ParseIntError),
    StatusError,
    FreqError,
    BydayError,
}

impl From<ParserError> for EventError {
    fn from(err: ParserError) -> EventError {
        EventError::IcalError(err)
    }
}

impl From<ParseIntError> for EventError {
    fn from(err: ParseIntError) -> EventError {
        EventError::IntError(err)
    }
}

