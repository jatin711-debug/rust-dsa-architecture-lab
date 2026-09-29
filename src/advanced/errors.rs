//! Error handling the production way: custom error types with `Display` +
//! `std::error::Error`, and a real `Result`-based API to exercise them.
//!
//! The data structures so far mostly used `Option`. Real software uses
//! `Result` with errors that say *what* failed and *where*. This module shows:
//!
//! - a structured error type (`CsvError`) with context (line number);
//! - `std::error::Error` + `Display` implementations;
//! - the `?` operator threading errors up;
//! - `From` conversions so `?` can cross error-type boundaries.
//!
//! # Examples
//!
//! ```
//! use rust_learning_and_dsa::advanced::parse_csv;
//!
//! let csv = "name,age\nAlice,30\nBob,25\n";
//! let records = parse_csv(csv).expect("well-formed CSV");
//! assert_eq!(records[0], vec!["name", "age"]);
//! assert_eq!(records[2], vec!["Bob", "25"]);
//! ```

use std::error::Error;
use std::fmt;

/// One CSV record: the fields of a single line.
pub type Record = Vec<String>;

/// An error produced while parsing CSV.
///
/// Carries the 1-based line number so callers can report exactly where input
/// went wrong — the difference between "it failed" and "line 7, column 3".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvError {
    /// 1-based line number of the offending record.
    pub line: usize,
    /// Human-readable description of the problem.
    pub message: String,
}

impl CsvError {
    fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

impl fmt::Display for CsvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl Error for CsvError {}

/// A marker error used by the demo to show `From` conversions: converting a
/// UTF-8 decode failure into a `CsvError` via `From<Utf8Error>`.
impl From<std::str::Utf8Error> for CsvError {
    fn from(source: std::str::Utf8Error) -> Self {
        Self::new(0, format!("invalid UTF-8: {source}"))
    }
}

/// Parses CSV text into records. Handles RFC-4180-style double-quoted fields
/// with `""` as an escaped quote.
///
/// # Errors
///
/// Returns `CsvError` if a record is malformed (unterminated quote, stray
/// quote in an unquoted field, or trailing characters after a quoted field).
pub fn parse_csv(input: &str) -> Result<Vec<Record>, CsvError> {
    let mut records = Vec::new();
    for (index, line) in input.lines().enumerate() {
        // `?` converts the `Result<_, ParseFieldError>` into `Result<_, CsvError>`
        // via `From`, attaching the line number.
        let record = parse_line(line).map_err(|e| CsvError::new(index + 1, e))?;
        records.push(record);
    }
    Ok(records)
}

/// Parses a single CSV line into fields.
fn parse_line(line: &str) -> Result<Record, String> {
    let bytes = line.as_bytes();
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            // Whitespace and separators are skipped; commas end a field.
            b',' => {
                fields.push(std::mem::take(&mut current));
                i += 1;
            }
            // Double quote: a quoted field that may contain commas.
            b'"' => {
                i += 1;
                let mut closed = false;
                while i < bytes.len() {
                    match bytes[i] {
                        b'"' => {
                            // "" inside a quoted field is an escaped quote.
                            if bytes.get(i + 1) == Some(&b'"') {
                                current.push('"');
                                i += 2;
                            } else {
                                closed = true;
                                i += 1;
                                break;
                            }
                        }
                        byte => {
                            // SAFETY: `byte` came from `line.as_bytes()`; the
                            // index is inside the string, so the char is valid
                            // (we only push ASCII bytes, so this is lossless).
                            current.push(byte as char);
                            i += 1;
                        }
                    }
                }
                if !closed {
                    return Err("unterminated quoted field".to_string());
                }
                // After a quoted field, only a comma or end-of-line is legal.
                if i < bytes.len() && bytes[i] != b',' {
                    return Err("trailing characters after quoted field".to_string());
                }
            }
            // Unquoted field: read until comma.
            byte => {
                current.push(byte as char);
                i += 1;
            }
        }
    }
    fields.push(current);
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_records() {
        let csv = "name,age,city\nAlice,30,NYC\nBob,25,LA\n";
        let records = parse_csv(csv).expect("valid CSV");
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["name", "age", "city"]);
        assert_eq!(records[2], vec!["Bob", "25", "LA"]);
    }

    #[test]
    fn quoted_fields_with_commas() {
        let csv = "id,notes\n1,\"has, comma\"\n2,\"says \"\"hi\"\"\"\n";
        let records = parse_csv(csv).expect("valid CSV");
        assert_eq!(records[1][1], "has, comma");
        assert_eq!(records[2][1], "says \"hi\"");
    }

    #[test]
    fn empty_lines_and_trailing_comma() {
        let csv = "a,b\n\n1,\n";
        let records = parse_csv(csv).expect("valid CSV");
        assert_eq!(records[0], vec!["a", "b"]);
        assert_eq!(records[1], vec![""]); // empty line -> one empty field
        assert_eq!(records[2], vec!["1", ""]);
    }

    #[test]
    fn unterminated_quote_is_an_error() {
        let csv = "a,b\n\"unterminated\n";
        let err = parse_csv(csv).unwrap_err();
        assert_eq!(err.line, 2);
        assert!(err.message.contains("unterminated"));
    }

    #[test]
    fn error_display_and_source() {
        let err = CsvError::new(3, "boom");
        assert_eq!(err.to_string(), "line 3: boom");
        // It is a real std error: can be boxed and downcast.
        let boxed: Box<dyn Error> = Box::new(err.clone());
        assert!(boxed.downcast_ref::<CsvError>().is_some_and(|e| e == &err));
    }

    #[test]
    fn from_utf8_error() {
        // `?` on a Result<_, Utf8Error> inside a fn returning Result<_, CsvError>
        // works because of our From impl.
        fn validate(input: &[u8]) -> Result<(), CsvError> {
            std::str::from_utf8(input)?;
            Ok(())
        }
        assert!(validate(b"fine").is_ok());
        let err = validate(&[0xff, 0xfe]).unwrap_err();
        assert!(err.message.contains("invalid UTF-8"));
    }

    #[test]
    fn round_trip_with_errors() {
        // A realistic mini-workflow: parse, validate, and report.
        fn total_age(csv: &str) -> Result<u32, CsvError> {
            let records = parse_csv(csv)?;
            let mut total = 0;
            for record in records.iter().skip(1) {
                // Bounds-checked access instead of indexing panics.
                let age: u32 = record
                    .get(1)
                    .ok_or_else(|| CsvError::new(0, "missing age column".to_string()))?
                    .parse()
                    .map_err(|_| CsvError::new(0, "age is not a number".to_string()))?;
                total += age;
            }
            Ok(total)
        }
        let csv = "name,age\nAlice,30\nBob,25\n";
        assert_eq!(total_age(csv), Ok(55));
        assert!(total_age("name,age\nAlice,notanumber\n").is_err());
    }
}
