//! STEP/IFC parser built with [nom](https://docs.rs/nom).
//!
//! Provides zero-copy tokenization of individual STEP entity lines.
//! The public entry point is [`parse_entity`], which takes a single
//! entity line (e.g. `#123=IFCWALL('guid',$,...);`) and returns the
//! entity id, type, and attribute tokens.

use nom::{
    branch::alt,
    bytes::complete::{take_while, take_while1},
    character::complete::{char, digit1, one_of},
    combinator::{map, map_res, opt, recognize},
    multi::separated_list0,
    sequence::{delimited, pair, preceded, tuple},
    IResult,
};

use crate::entity_id::EntityId;
use crate::error::{Error, Result};
use crate::ifc_type::IfcType;
use crate::token::Token;

// ═══════════════════════════════════════════════════════════════════
// Individual token parsers
// ═══════════════════════════════════════════════════════════════════

/// `#123`
fn entity_ref(input: &str) -> IResult<&str, Token<'_>> {
    map(
        preceded(char('#'), map_res(digit1, |s: &str| s.parse::<u32>())),
        |id| Token::EntityRef(EntityId::new(id)),
    )(input)
}

/// `'hello'` or `"hello"`, with `''` / `""` escape handling via memchr.
fn string_literal(input: &str) -> IResult<&str, Token<'_>> {
    #[inline]
    fn content_up_to_unescaped_quote(input: &str, quote: u8) -> IResult<&str, &str> {
        let bytes = input.as_bytes();
        let end = find_unescaped_quote(bytes, quote, 0);
        end.map_or(
            Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Char,
            ))),
            |idx| Ok((&input[idx..], &input[..idx])),
        )
    }

    alt((
        map(
            delimited(
                char('\''),
                |i| content_up_to_unescaped_quote(i, b'\''),
                char('\''),
            ),
            Token::Str,
        ),
        map(
            delimited(
                char('"'),
                |i| content_up_to_unescaped_quote(i, b'"'),
                char('"'),
            ),
            Token::Str,
        ),
    ))(input)
}

/// Walk `bytes` starting at `pos` looking for `quote` that is not doubled.
fn find_unescaped_quote(bytes: &[u8], quote: u8, pos: usize) -> Option<usize> {
    memchr::memchr(quote, bytes.get(pos..)?).and_then(|offset| {
        let idx = pos + offset;
        bytes
            .get(idx + 1)
            .filter(|&&next| next == quote)
            .map_or(Some(idx), |_| find_unescaped_quote(bytes, quote, idx + 2))
    })
}

/// `42`, `-42`
fn integer(input: &str) -> IResult<&str, Token<'_>> {
    map_res(recognize(tuple((opt(char('-')), digit1))), |s: &str| {
        s.parse::<i64>().map(Token::Integer)
    })(input)
}

/// `3.14`, `-3.14`, `1.5E-10`, `0.`
fn float(input: &str) -> IResult<&str, Token<'_>> {
    map_res(
        recognize(tuple((
            opt(char('-')),
            digit1,
            char('.'),
            opt(digit1),
            opt(tuple((one_of("eE"), opt(one_of("+-")), digit1))),
        ))),
        |s: &str| s.parse::<f64>().map(Token::Float),
    )(input)
}

/// `.TRUE.`, `.ELEMENT.`
fn enum_value(input: &str) -> IResult<&str, Token<'_>> {
    map(
        delimited(
            char('.'),
            take_while1(|c: char| c.is_alphanumeric() || c == '_'),
            char('.'),
        ),
        Token::Enum,
    )(input)
}

/// `$`
fn null(input: &str) -> IResult<&str, Token<'_>> {
    map(char('$'), |_| Token::Null)(input)
}

/// `*`
fn derived(input: &str) -> IResult<&str, Token<'_>> {
    map(char('*'), |_| Token::Derived)(input)
}

/// `IFCPARAMETERVALUE(0.)`, `IFCBOOLEAN(.T.)`
fn typed_value(input: &str) -> IResult<&str, Token<'_>> {
    map(
        pair(
            take_while1(|c: char| c.is_alphanumeric() || c == '_'),
            delimited(
                char('('),
                separated_list0(delimited(ws, char(','), ws), token),
                char(')'),
            ),
        ),
        |(name, args)| Token::TypedValue(name, args),
    )(input)
}

/// Skip optional whitespace.
fn ws(input: &str) -> IResult<&str, ()> {
    map(take_while(|c: char| c.is_whitespace()), |_| ())(input)
}

/// Parse a single token (with surrounding whitespace stripped).
///
/// Ordered cheapest-first: single-char markers, then enums/strings,
/// then numbers, then the most expensive typed-value match.
fn token(input: &str) -> IResult<&str, Token<'_>> {
    delimited(
        ws,
        alt((
            null,
            derived,
            entity_ref,
            enum_value,
            string_literal,
            list,
            float,
            integer,
            typed_value,
        )),
        ws,
    )(input)
}

/// `(1, 2, 3)` or nested lists.
fn list(input: &str) -> IResult<&str, Token<'_>> {
    map(
        delimited(
            char('('),
            separated_list0(delimited(ws, char(','), ws), token),
            char(')'),
        ),
        Token::List,
    )(input)
}

// ═══════════════════════════════════════════════════════════════════
// Public API
// ═══════════════════════════════════════════════════════════════════

/// Parse a complete STEP entity line.
///
/// Expects the full line including the trailing `;`.
///
/// # Errors
///
/// Returns [`Error::Parse`] when the input does not conform to STEP
/// entity syntax.
///
/// # Examples
///
/// ```
/// use ifc_lite_core::{parse_entity, EntityId, IfcType};
///
/// let line = "#123=IFCWALL('guid',$,$,$,'Wall',$,$,$);";
/// let (id, ifc_type, attrs) = parse_entity(line).unwrap();
/// assert_eq!(id, EntityId::new(123));
/// assert_eq!(ifc_type, IfcType::IfcWall);
/// assert_eq!(attrs.len(), 8);
/// ```
pub fn parse_entity(input: &str) -> Result<(EntityId, IfcType, Vec<Token<'_>>)> {
    let result: IResult<&str, (u32, &str, Vec<Token<'_>>)> = tuple((
        delimited(
            ws,
            preceded(char('#'), map_res(digit1, |s: &str| s.parse::<u32>())),
            ws,
        ),
        preceded(
            char('='),
            delimited(
                ws,
                take_while1(|c: char| c.is_alphanumeric() || c == '_'),
                ws,
            ),
        ),
        delimited(
            char('('),
            separated_list0(delimited(ws, char(','), ws), token),
            tuple((char(')'), ws, char(';'))),
        ),
    ))(input);

    result
        .map(|(_, (id, type_str, args))| (EntityId::new(id), IfcType::from_name(type_str), args))
        .map_err(|e| Error::parse(0, format!("failed to parse entity: {e}")))
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_entity_ref() {
        assert_eq!(entity_ref("#123"), Ok(("", Token::EntityRef(EntityId::new(123)))));
        assert_eq!(entity_ref("#0"), Ok(("", Token::EntityRef(EntityId::new(0)))));
    }

    #[test]
    fn parse_string_literal() {
        assert_eq!(string_literal("'hello'"), Ok(("", Token::Str("hello"))));
        assert_eq!(string_literal("'with spaces'"), Ok(("", Token::Str("with spaces"))));
    }

    #[test]
    fn parse_integer_values() {
        assert_eq!(integer("42"), Ok(("", Token::Integer(42))));
        assert_eq!(integer("-42"), Ok(("", Token::Integer(-42))));
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn parse_float_values() {
        assert_eq!(float("3.14"), Ok(("", Token::Float(3.14))));
        assert_eq!(float("-3.14"), Ok(("", Token::Float(-3.14))));
        assert_eq!(float("0."), Ok(("", Token::Float(0.0))));
    }

    #[test]
    fn parse_enum_values() {
        assert_eq!(enum_value(".TRUE."), Ok(("", Token::Enum("TRUE"))));
        assert_eq!(enum_value(".ELEMENT."), Ok(("", Token::Enum("ELEMENT"))));
    }

    #[test]
    fn parse_list_values() {
        let (_, tok) = list("(1,2,3)").expect("list parse");
        match tok {
            Token::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], Token::Integer(1));
            }
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn parse_full_entity() {
        let input = "#123=IFCWALL('guid','owner',$,$,'name',$,$,$);";
        let (id, ifc_type, args) = parse_entity(input).expect("parse_entity");
        assert_eq!(id, EntityId::new(123));
        assert_eq!(ifc_type, IfcType::IfcWall);
        assert_eq!(args.len(), 8);
    }

    #[test]
    fn parse_entity_with_nested_list() {
        let input = "#9=IFCDIRECTION((0.,0.,1.));";
        let (id, _, args) = parse_entity(input).expect("parse_entity");
        assert_eq!(id, EntityId::new(9));
        assert_eq!(args.len(), 1);
        match &args[0] {
            Token::List(inner) => assert_eq!(inner.len(), 3),
            other => panic!("expected List, got {other:?}"),
        }
    }
}
