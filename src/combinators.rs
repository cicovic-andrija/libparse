//! Generic parser combinators

use crate::base::*;

/// Parser generator for parsing a pair of tokens and returning results as a tuple
pub fn pair<P1, P2, I, O1, O2>(
    mut left_parser: P1,
    mut right_parser: P2,
) -> impl Parser<I, (O1, O2)>
where
    P1: Parser<I, O1>,
    P2: Parser<I, O2>,
    I: Copy,
{
    move |input: I| {
        left_parser.parse(input).and_then(|(next_input, left)| {
            right_parser
                .parse(next_input)
                .map_err(|err| Error::new(input, err.code))
                .map(|(rem_input, right)| (rem_input, (left, right)))
        })
    }
}

/// Parser generator for parsing a pair of tokens and returning only the left result
pub fn left_from_pair<P1, P2, I, O1, O2>(left_parser: P1, right_parser: P2) -> impl Parser<I, O1>
where
    P1: Parser<I, O1>,
    P2: Parser<I, O2>,
    I: Copy,
{
    pair(left_parser, right_parser).map(|(left, _)| left)
}

/// Parser generator for parsing a pair of tokens and returning only the right result
pub fn right_from_pair<P1, P2, I, O1, O2>(left_parser: P1, right_parser: P2) -> impl Parser<I, O2>
where
    P1: Parser<I, O1>,
    P2: Parser<I, O2>,
    I: Copy,
{
    pair(left_parser, right_parser).map(|(_, right)| right)
}

/// Parser generator for parsing zero or more occurrences of a token
pub fn zero_or_more<P, I, O>(mut parser: P) -> impl Parser<I, Vec<O>>
where
    P: Parser<I, O>,
{
    move |mut input: I| {
        let mut outputs = Vec::new();
        let err = loop {
            match parser.parse(input) {
                Ok((next_input, next_output)) => {
                    input = next_input;
                    outputs.push(next_output);
                }
                Err(err) => break err,
            }
        };

        if err.is_failure() {
            Err(err)
        } else {
            Ok((err.input, outputs))
        }
    }
}
