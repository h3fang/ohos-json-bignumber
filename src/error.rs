use std::fmt;

use napi_ohos::{Error, Status};

#[derive(Debug)]
pub enum ParseError {
    UnexpectedCharacter(char),
    UnexpectedEndOfInput,
    InvalidNumber,
    InvalidEscapeSequence(char),
    ExpectedColon,
    ExpectedCommaOrEnd,
    TrailingCharacters,
    NapiError(Error),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl AsRef<str> for ParseError {
    fn as_ref(&self) -> &str {
        match self {
            ParseError::UnexpectedCharacter(_) => "UnexpectedCharacter",
            ParseError::UnexpectedEndOfInput => "UnexpectedEndOfInput",
            ParseError::InvalidNumber => "InvalidNumber",
            ParseError::InvalidEscapeSequence(_) => "InvalidEscapeSequence",
            ParseError::ExpectedColon => "ExpectedColon",
            ParseError::ExpectedCommaOrEnd => "ExpectedCommaOrEnd",
            ParseError::TrailingCharacters => "TrailingCharacters",
            ParseError::NapiError(error) => error.status.as_ref(),
        }
    }
}

impl From<Error> for ParseError {
    fn from(err: Error) -> Self {
        ParseError::NapiError(err)
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        match err {
            ParseError::NapiError(e) => e,
            _ => Error::new(Status::GenericFailure, err),
        }
    }
}
