//! Core types needed to build a parser

use std::marker::{PhantomData, Sized};

/// Code that indicates where parsing failed
#[derive(Debug, PartialEq, Eq)]
pub enum ErrorCode {
    Failure(Reason),
    NoInput,
    Char(char),
    LineBreak,
    Predicate,
}

/// Describes Failure reason
#[derive(Debug, PartialEq, Eq)]
pub enum Reason {
    _SystemFailure,
    InvalidInput { expected: &'static str },
}

/// Generic parsing error
#[derive(Debug, PartialEq, Eq)]
pub struct Error<I> {
    pub input: I,
    pub code: ErrorCode,
}

impl<I> Error<I> {
    /// Creates a new error with the given error code
    pub fn new(input: I, code: ErrorCode) -> Self {
        Error { input, code }
    }

    /// Creates a new error that indicates failure
    pub fn failure(input: I, reason: Reason) -> Self {
        Error {
            input,
            code: ErrorCode::Failure(reason),
        }
    }

    /// Indicates whether this error is a failure
    pub fn is_failure(&self) -> bool {
        match self.code {
            ErrorCode::Failure(_) => true,
            _ => false,
        }
    }
}

/// Result type of parsing
///
/// When parsing results in `Ok`, contains the remainder of the input
/// and the output result of the parsing
pub type PResult<I, O> = Result<(I, O), Error<I>>;

/// All parsers should implement this trait
pub trait Parser<I, O> {
    /// Parses an input type and returns an output type of a parsing error
    fn parse(&mut self, input: I) -> PResult<I, O>;

    /// Moves this parser to a new one that applies a map function on the result
    fn map<F, O2>(self, map_fn: F) -> Map<Self, F, O>
    where
        F: FnMut(O) -> O2,
        Self: Sized,
    {
        Map {
            parser: self,
            map_fn,
            phantom: PhantomData,
        }
    }

    /// Moves this parser to a new one that will apply a map function on the result of parsing
    /// to produce a new parser for the following input
    fn and_then_map<F, P2, O2>(self, map_fn: F) -> AndThenMap<Self, F, O, P2>
    where
        F: FnMut(O) -> P2,
        P2: Parser<I, O2>,
        Self: Sized,
    {
        AndThenMap {
            first: self,
            map_fn,
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }

    /// Moves this parser to a new one that can fallback to a given parser in case this one fails
    fn fallback_on<P2>(self, fallback_parser: P2) -> Fallback<Self, P2>
    where
        P2: Parser<I, O>,
        Self: Sized,
    {
        Fallback {
            primary: self,
            fallback: fallback_parser,
        }
    }

    /// Moves this parser to a new one that accepts the results of parsing only if it satisfies the given predicate
    fn iff<F>(self, predicate: F) -> Predicate<Self, F>
    where
        F: FnMut(&O) -> bool,
        I: Copy,
        Self: Sized,
    {
        Predicate {
            parser: self,
            predicate,
            assume_invalid: false,
        }
    }

    /// Moves this parser to a new one that accepts the results of parsing only if it satisfies the given predicate,
    /// otherwise it assumes invalid input and reports that parsing has failed
    fn iff_or_invalid<F>(self, predicate: F) -> Predicate<Self, F>
    where
        F: FnMut(&O) -> bool,
        I: Copy,
        Self: Sized,
    {
        Predicate {
            parser: self,
            predicate,
            assume_invalid: true,
        }
    }
}

impl<I, O, F> Parser<I, O> for F
where
    F: FnMut(I) -> PResult<I, O>,
{
    fn parse(&mut self, input: I) -> PResult<I, O> {
        self(input)
    }
}

/// Map is a parser that applies a map function on the result of parsing
pub struct Map<P, F, O1> {
    parser: P,
    map_fn: F,
    phantom: PhantomData<O1>,
}

impl<I, O1, O2, P, F> Parser<I, O2> for Map<P, F, O1>
where
    P: Parser<I, O1>,
    F: FnMut(O1) -> O2,
{
    fn parse(&mut self, input: I) -> PResult<I, O2> {
        match self.parser.parse(input) {
            Err(e) => Err(e),
            Ok((next_input, result)) => Ok((next_input, (self.map_fn)(result))),
        }
    }
}

/// AndThenMap is a parser that applies map function on the result of parsing to produce a new parser
/// for the following input
pub struct AndThenMap<P1, F, O1, P2> {
    first: P1,
    map_fn: F,
    phantom1: PhantomData<O1>,
    phantom2: PhantomData<P2>,
}

impl<I, O1, O2, P1, P2, F> Parser<I, O2> for AndThenMap<P1, F, O1, P2>
where
    P1: Parser<I, O1>,
    P2: Parser<I, O2>,
    F: FnMut(O1) -> P2,
{
    fn parse(&mut self, input: I) -> PResult<I, O2> {
        match self.first.parse(input) {
            Ok((next_input, result)) => (self.map_fn)(result).parse(next_input),
            Err(err) => Err(err),
        }
    }
}

/// Fallback is a parser that applies a fallback parsing logic in case the primary one fails
pub struct Fallback<P1, P2> {
    primary: P1,
    fallback: P2,
}

impl<I, O, P1, P2> Parser<I, O> for Fallback<P1, P2>
where
    P1: Parser<I, O>,
    P2: Parser<I, O>,
{
    fn parse(&mut self, input: I) -> PResult<I, O> {
        self.primary
            .parse(input)
            .or_else(|err: Error<I>| self.fallback.parse(err.input))
    }
}

/// Accepts the results of parsing only if it satisfies the given predicate
pub struct Predicate<P, F> {
    parser: P,
    predicate: F,
    assume_invalid: bool,
}

impl<I, O, P, F> Parser<I, O> for Predicate<P, F>
where
    P: Parser<I, O>,
    F: FnMut(&O) -> bool,
    I: Copy,
{
    fn parse(&mut self, input: I) -> PResult<I, O> {
        let (next_input, result) = self.parser.parse(input)?;
        if (self.predicate)(&result) {
            Ok((next_input, result))
        } else {
            if self.assume_invalid {
                Err(Error::failure(
                    input,
                    Reason::InvalidInput {
                        expected: "unknown",
                    },
                ))
            } else {
                Err(Error::new(input, ErrorCode::Predicate))
            }
        }
    }
}
