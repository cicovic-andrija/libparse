//! Parser functional tests

#[cfg(test)]
mod tests {
    use crate::chars::*;
    use crate::csv;
    use crate::*;

    fn any_char(input: &str) -> PResult<&str, char> {
        match input.chars().next() {
            Some(ch) => Ok((&input[ch.len_utf8()..], ch)),
            _ => Err(Error::new(input, END_OF_STRING)),
        }
    }

    #[test]
    fn simple_parser() {
        let input = "abc";
        let (input, ch) = any_char.parse(input).unwrap();
        assert_eq!(ch, 'a');
        let (input, ch) = any_char.parse(input).unwrap();
        assert_eq!(ch, 'b');
        let (input, ch) = any_char.parse(input).unwrap();
        assert_eq!(ch, 'c');
        assert_eq!(any_char.parse(input), Err(Error::new(input, END_OF_STRING)));
    }

    #[test]
    fn simple_parser_map() {
        let input = "abc";
        let mut map = any_char.map(|ch| ch.to_digit(16).unwrap());
        let (input, num) = map.parse(input).unwrap();
        assert_eq!(num, 10);
        assert_eq!(input, "bc");
        let (input, num) = map.parse(input).unwrap();
        assert_eq!(num, 11);
        assert_eq!(input, "c");
        let (input, num) = map.parse(input).unwrap();
        assert_eq!(num, 12);
        assert_eq!(any_char.parse(input), Err(Error::new(input, END_OF_STRING)));
    }

    #[test]
    fn line_break_parser() {
        assert_eq!(line_break.parse("\n"), Ok(("", "\n")));
        assert_eq!(line_break.parse("\r\n"), Ok(("", "\r\n")));
        assert_eq!(
            line_break.parse("\nsecond line\n"),
            Ok(("second line\n", "\n"))
        );
        assert_eq!(
            line_break.parse("\r\nsecond line\r\n"),
            Ok(("second line\r\n", "\r\n"))
        );
        assert_eq!(
            line_break.parse("not an end of this line\n"),
            Err(Error::new(
                "not an end of this line\n",
                ErrorCode::LineBreak
            ))
        );
        assert_eq!(
            line_break.parse("\r"),
            Err(Error::new("\r", ErrorCode::LineBreak))
        );
    }

    #[test]
    fn empty_string() {
        let (input, ch) = any_char.parse("a").unwrap();
        assert_eq!(ch, 'a');
        assert_eq!(input.len(), 0);
        assert_eq!(input, "");
        let result = any_char.parse(input);
        assert_eq!(result, Err(Error::new("", END_OF_STRING)));
        let result = any_char.parse(input);
        assert_eq!(result, Err(Error::new("", END_OF_STRING)));
    }

    #[test]
    fn pair_combinator() {
        let mut combi = pair(char('a'), char('b'));
        let (rem_input, (left, right)) = combi.parse("abc").unwrap();
        assert_eq!(left, 'a');
        assert_eq!(right, 'b');
        assert_eq!(rem_input, "c");
    }

    #[test]
    fn pari_combinator_negative() {
        let mut combi = pair(char('a'), char('b'));
        assert_eq!(
            combi.parse("acb"),
            Err(Error::new("acb", ErrorCode::Char('b'))),
        )
    }

    #[test]
    fn left_combinator() {
        let mut combi = left_from_pair(char('a'), line_break);
        let (_, left) = combi.parse("a\r\n").unwrap();
        assert_eq!(left, 'a');
    }

    #[test]
    fn right_combinator() {
        let mut combi = right_from_pair(char(','), char('b'));
        let (_, right) = combi.parse(",b").unwrap();
        assert_eq!(right, 'b');
    }

    #[test]
    fn zero_or_more_combinator_consume_part() {
        let input = "aaabc";
        let mut combi = zero_or_more(char('a'));
        let (input, outputs) = combi.parse(input).unwrap();
        assert_eq!(outputs.len(), 3);
        assert_eq!(input, "bc");
        for ch in outputs {
            assert_eq!(ch, 'a');
        }
        let (input, outputs) = combi.parse(input).unwrap();
        assert_eq!(outputs.len(), 0);
        assert_eq!(input, "bc");
    }

    #[test]
    fn zero_or_more_combinator_consume_all() {
        let mut combi = zero_or_more(any_char);
        let (input, output) = combi.parse("aaabc").unwrap();
        assert_eq!(input, "");
        assert_eq!(output.into_iter().collect::<String>(), "aaabc");
        assert_eq!(combi.parse(input), Ok(("", Vec::new())));
    }

    #[test]
    fn predicate_combinator() {
        let mut combi = any_char.iff(|ch| *ch == 'a');
        let (input, ch) = combi.parse("abc").unwrap();
        assert_eq!(ch, 'a');
        assert_eq!(
            combi.parse(input),
            Err(Error::new(input, ErrorCode::Predicate))
        )
    }

    #[test]
    fn csv_field_parser_empty() {
        let mut comma = char(',');
        let (next_input, field) = csv::field(",,").unwrap();
        assert_eq!(next_input, ",,");
        assert_eq!(field, "");
        let (next_input, _) = comma.parse(next_input).unwrap();
        assert_eq!(next_input, ",");
        let (next_input, field) = csv::field(next_input).unwrap();
        assert_eq!(next_input, ",");
        assert_eq!(field, "");
        let (next_input, _) = comma.parse(next_input).unwrap();
        assert_eq!(next_input, "");
        let (next_input, field) = csv::field(next_input).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(field, "");
    }

    #[test]
    fn csv_field_parser_non_escaped() {
        let (next_input, field) = csv::field.parse("field1").unwrap();
        assert_eq!(next_input, "");
        assert_eq!(field, "field1");

        let (next_input, field) = csv::field.parse("  \tfield1,field2").unwrap();
        assert_eq!(next_input, ",field2");
        assert_eq!(field, "  \tfield1");

        let (next_input, field) = csv::field.parse(",field2").unwrap();
        assert_eq!(next_input, ",field2");
        assert_eq!(field, "");

        let (next_input, field) = csv::field.parse("").unwrap();
        assert_eq!(next_input, "");
        assert_eq!(field, "");

        let (next_input, field) = csv::field.parse("test\"quote").unwrap();
        assert_eq!(next_input, "\"quote");
        assert_eq!(field, "test");
    }

    #[test]
    fn csv_field_parser_escaped() {
        let (next_input, field) = csv::field.parse("\"field1\"").unwrap();
        assert_eq!(next_input, "");
        assert_eq!(field, "field1");

        let (next_input, field) = csv::field.parse("\"with,comma\",next").unwrap();
        assert_eq!(next_input, ",next");
        assert_eq!(field, "with,comma");

        let (next_input, field) = csv::field
            .parse("\"with\rspecial\ncharacters\r\n\"")
            .unwrap();
        assert_eq!(next_input, "");
        assert_eq!(field, "with\rspecial\ncharacters\r\n");

        let (next_input, field) = csv::field.parse("\"with \"\"double quotes\"\"\"").unwrap();
        assert_eq!(next_input, "");
        assert_eq!(field, "with \"double quotes\"");

        let (next_input, field) = csv::field.parse(
            "\"a\tvery\"\"complex\"\" example, including\nmultiple special characters\r\n\",and another field",
        )
        .unwrap();
        assert_eq!(next_input, ",and another field");
        assert_eq!(
            field,
            "a\tvery\"complex\" example, including\nmultiple special characters\r\n"
        );
    }

    #[test]
    fn csv_record_parser() {
        let (next_input, fields) = csv::record.parse("field1,field2,field3").unwrap();
        assert_eq!(next_input, "");
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0], "field1");
        assert_eq!(fields[1], "field2");
        assert_eq!(fields[2], "field3");

        let (next_input, fields) = csv::record.parse("the only field").unwrap();
        assert_eq!(next_input, "");
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0], "the only field");

        assert_eq!(
            csv::record.parse(""),
            Err(Error::new("", ErrorCode::NoInput)),
        );

        let (next_input, fields) = csv::record
            .parse("107,\"\"\"Pogledaj dom svoj, anđele\"\"\",,Tomas Vulf,Roman,sr,en,,Ne,,1\r\n")
            .unwrap();
        assert_eq!(next_input, "\r\n");
        assert_eq!(fields.len(), 11);
        assert_eq!(fields[0], "107");
        assert_eq!(fields[1], "\"Pogledaj dom svoj, anđele\"");
        assert_eq!(fields[2], "");
        assert_eq!(fields[3], "Tomas Vulf");
        assert_eq!(fields[4], "Roman");
        assert_eq!(fields[5], "sr");
        assert_eq!(fields[6], "en");
        assert_eq!(fields[7], "");
        assert_eq!(fields[8], "Ne");
        assert_eq!(fields[9], "");
        assert_eq!(fields[10], "1");
    }

    #[test]
    fn csv_document_parser() {
        let input = concat!(
            "\"\"\"The Fellowship of the Ring\"\"\",J. R. R. Tolkien,en,HarperCollins Illustrated Hardback\r\n",
            "\"\"\"The Two Towers\"\"\",J. R. R. Tolkien,en,HarperCollins Illustrated Hardback\r\n",
            "\"\"\"The Return of the King\"\"\",J. R. R. Tolkien,en,HarperCollins Illustrated Hardback\r\n"
        );

        let (next_input, records) = csv::parse_string(input).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(records.len(), 3);

        assert_eq!(records[0][0], "\"The Fellowship of the Ring\"");
        assert_eq!(records[0][1], "J. R. R. Tolkien");
        assert_eq!(records[0][2], "en");
        assert_eq!(records[0][3], "HarperCollins Illustrated Hardback");

        assert_eq!(records[1][0], "\"The Two Towers\"");
        assert_eq!(records[1][1], "J. R. R. Tolkien");
        assert_eq!(records[1][2], "en");
        assert_eq!(records[1][3], "HarperCollins Illustrated Hardback");

        assert_eq!(records[2][0], "\"The Return of the King\"");
        assert_eq!(records[2][1], "J. R. R. Tolkien");
        assert_eq!(records[2][2], "en");
        assert_eq!(records[2][3], "HarperCollins Illustrated Hardback");

        let input = concat!(
            "\"\"\"Hyperion\"\"\",Dan Simmons\n",
            "\"\"\"The Fall of Hyperion\"\"\"",
        );

        assert_eq!(
            csv::parse_string(input),
            Err(Error::failure(
                "\n\"\"\"The Fall of Hyperion\"\"\"",
                Reason::InvalidInput {
                    expected: "more fields in this record"
                },
            )),
        );
    }

    #[test]
    fn csv_document_parser_from_file() {
        let input = std::fs::read_to_string("src/test_data/books.csv").unwrap();
        let (next_input, records) = csv::parse_string(input.as_str()).unwrap();
        assert_eq!(next_input, "");
        assert_eq!(records.len(), 30);
        for rec in records {
            assert_eq!(rec.len(), 11);
        }
    }
}
