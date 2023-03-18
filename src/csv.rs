//! CSV-related parsers
//!
//! CSV EBNF grammar, derived from [RFC4180](https://www.rfc-editor.org/rfc/rfc4180)
//!
//! CSVDATA = RECORD (LINEBREAK RECORD)* [LINEBREAK]
//! RECORD = FIELD (COMMA FIELD)*
//! FIELD = ESCAPED | NON-ESCAPED
//! ESCAPED = DQUOTE (TEXT | COMMA | CR | LF | DDQUOTE)* DQUOTE
//! NON-ESCAPED = TEXT*
//! CR = "\r" (* %x2C *)
//! LF = "\n" (* %x0A *)
//! COMMA = "," (* %x2C *)
//! DQUOTE = """ (* %x22 *)
//! DDQUOTE = """"
//! LINEBREAK = LF | CRLF
//! (* TEXT is any encoding that does not encode CR, LF, COMMA or DQUOTE *)

use crate::base::*;
use crate::chars::*;
use crate::combinators::*;

pub fn parse_string(input: &str) -> PResult<&str, Vec<CsvRecord>> {
    let mut records: Vec<CsvRecord> = Vec::new();
    let (trailing, records) = record
        .and_then_map(|first_record| {
            let len = first_record.len();
            records.push(first_record);
            zero_or_more(right_from_pair(
                line_break,
                record.iff_or_invalid(move |rec| rec.len() == len),
            ))
        })
        .parse(input)
        .map_err(|err| match err {
            Error {
                input,
                code: ErrorCode::Failure(Reason::InvalidInput { .. }),
            } => Error::failure(
                input,
                Reason::InvalidInput {
                    expected: "more fields in this record",
                },
            ),
            err => err,
        })
        .and_then(|(rem_input, other_records)| {
            records.extend(other_records);
            Ok((rem_input, records))
        })?;

    // Parse optional line break at the end.
    if trailing.len() > 0 {
        match line_break.parse(trailing) {
            Ok(("", _))
            | Err(Error {
                input: "",
                code: ErrorCode::LineBreak,
            }) => Ok(("", records)),

            // Parser stumbled upon an invalid character or something is seriously wrong with
            // the parser implementation; assuming the first one
            _ => Err(Error::failure(
                trailing,
                Reason::InvalidInput {
                    expected: "comma or a line break",
                },
            )),
        }
    } else {
        Ok(("", records))
    }
}

/// Single CSV record (line) parser
pub type CsvRecord = Vec<String>;

/// Single CSV record parser
pub fn record(input: &str) -> PResult<&str, CsvRecord> {
    if input.len() > 0 {
        field.parse(input).and_then(|(next_input, first_field)| {
            let mut fields: CsvRecord = CsvRecord::new();
            fields.push(first_field);
            zero_or_more(right_from_pair(comma, field))
                .parse(next_input)
                .and_then(|(rem_input, other_fields)| {
                    fields.extend(other_fields);
                    Ok((rem_input, fields))
                })
        })
    } else {
        // Empty string is a valid record by CSV grammar, it's essentially a one empty field,
        // however this implementation does not allow it
        Err(Error::new(input, ErrorCode::NoInput))
    }
}

/// Single CSV field parser
pub fn field(input: &str) -> PResult<&str, String> {
    escaped.fallback_on(non_escaped).parse(input)
}

fn comma(input: &str) -> PResult<&str, char> {
    char(',')(input)
}

fn dquote(input: &str) -> PResult<&str, char> {
    char('"')(input)
}

fn is_special(ch: char) -> bool {
    ch == ',' || ch == '"' || ch == '\r' || ch == '\n'
}

fn non_escaped(input: &str) -> PResult<&str, String> {
    zero_or_more(any_char.iff(|ch| !is_special(*ch)))
        .map(|chars| chars.into_iter().collect())
        .parse(input)
}

fn escaped(input: &str) -> PResult<&str, String> {
    right_from_pair(
        dquote,
        left_from_pair(
            zero_or_more(
                any_char
                    .iff(|ch| *ch != '"')
                    .fallback_on(left_from_pair(dquote, dquote)),
            ),
            dquote,
        ),
    )
    .map(|chars| chars.into_iter().collect())
    .parse(input)
}
