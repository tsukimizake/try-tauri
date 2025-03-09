use std::sync::{Arc, Mutex};

use ne::ErrorKind;
use nom::combinator::opt;
use nom::error as ne;
use nom::{character::complete::space0, combinator::recognize};

use nom::{
    IResult,
    branch::alt,
    bytes::complete::{take_while, take_while1},
    character::complete::char,
    combinator::map,
    multi::many0,
    sequence::{delimited, pair, preceded, tuple},
};

use nom_locate::LocatedSpan;

pub type Value = super::super::elm_interface::Value;

use super::env::{Env, ModelId};
pub fn cast_evaled(expr: Arc<Expr>) -> Value {
    match expr.as_ref() {
        Expr::Integer { value, .. } => Value::Integer(*value),
        Expr::Double { value, .. } => Value::Double(*value),
        Expr::Model { id, .. } => Value::Stl(*id),
        Expr::String { value, .. } => Value::String(value.clone()),
        Expr::Symbol { name, .. } => Value::Symbol(name.clone()),
        Expr::List { elements, .. } => {
            Value::List(elements.iter().map(|e| cast_evaled(e.clone())).collect())
        }
        Expr::Quote { expr, .. } => cast_evaled(Arc::new((**expr).clone())),
        Expr::Quasiquote { expr, .. } => cast_evaled(Arc::new((**expr).clone())),
        Expr::Unquote { expr, .. } => cast_evaled(Arc::new((**expr).clone())),
        Expr::Builtin { name, .. } => Value::Symbol(format!("<builtin {}>", name)),
        Expr::SpecialForm { name, .. } => Value::Symbol(format!("<special form {}>", name)),
        Expr::Clausure { .. } => Value::Symbol("<closure>".to_string()),
        Expr::Macro { .. } => Value::Symbol("<macro>".to_string()),
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Symbol {
        name: String,
        location: Option<usize>,
        trailing_newline: bool,
    },
    List {
        elements: Vec<Arc<Expr>>,
        location: Option<usize>,
        trailing_newline: bool,
    },
    Integer {
        value: i64,
        location: Option<usize>,
        trailing_newline: bool,
    },
    String {
        value: String,
        location: Option<usize>,
        trailing_newline: bool,
    },
    Double {
        value: f64,
        location: Option<usize>,
        trailing_newline: bool,
    },
    Model {
        id: ModelId,
        location: Option<usize>,
        trailing_newline: bool,
    },
    Quote {
        expr: Box<Expr>,
        location: Option<usize>,
        trailing_newline: bool,
    },
    Quasiquote {
        expr: Box<Expr>,
        location: Option<usize>,
        trailing_newline: bool,
    },
    Unquote {
        expr: Box<Expr>,
        location: Option<usize>,
        trailing_newline: bool,
    },
    Builtin {
        name: String,
        fun: fn(&[Arc<Expr>], Arc<Mutex<Env>>) -> Result<Arc<Expr>, String>,
    },
    SpecialForm {
        name: String,
        fun: fn(&[Arc<Expr>], Arc<Mutex<Env>>) -> Result<Arc<Expr>, String>,
    },
    Clausure {
        args: Vec<String>,
        body: Arc<Expr>,
        env: Arc<Mutex<Env>>,
    },
    Macro {
        args: Vec<String>,
        body: Arc<Expr>,
        env: Arc<Mutex<Env>>,
    },
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        use Expr::*;
        match (self, other) {
            (
                Symbol {
                    name: n1,
                    location: loc1,
                    trailing_newline: tn1,
                },
                Symbol {
                    name: n2,
                    location: loc2,
                    trailing_newline: tn2,
                },
            ) => n1 == n2 && loc1 == loc2 && tn1 == tn2,

            (
                List {
                    elements: e1,
                    location: loc1,
                    trailing_newline: tn1,
                },
                List {
                    elements: e2,
                    location: loc2,
                    trailing_newline: tn2,
                },
            ) => e1 == e2 && loc1 == loc2 && tn1 == tn2,

            (
                Integer {
                    value: v1,
                    location: loc1,
                    trailing_newline: tn1,
                },
                Integer {
                    value: v2,
                    location: loc2,
                    trailing_newline: tn2,
                },
            ) => v1 == v2 && loc1 == loc2 && tn1 == tn2,

            (
                Double {
                    value: v1,
                    location: loc1,
                    trailing_newline: tn1,
                },
                Double {
                    value: v2,
                    location: loc2,
                    trailing_newline: tn2,
                },
            ) => v1 == v2 && loc1 == loc2 && tn1 == tn2,

            (
                String {
                    value: v1,
                    location: loc1,
                    trailing_newline: tn1,
                },
                String {
                    value: v2,
                    location: loc2,
                    trailing_newline: tn2,
                },
            ) => v1 == v2 && loc1 == loc2 && tn1 == tn2,

            (
                Model {
                    id: id1,
                    location: loc1,
                    trailing_newline: tn1,
                },
                Model {
                    id: id2,
                    location: loc2,
                    trailing_newline: tn2,
                },
            ) => id1 == id2 && loc1 == loc2 && tn1 == tn2,
            (
                Quote {
                    expr: e1,
                    location: loc1,
                    trailing_newline: tn1,
                },
                Quote {
                    expr: e2,
                    location: loc2,
                    trailing_newline: tn2,
                },
            ) => e1 == e2 && loc1 == loc2 && tn1 == tn2,
            (
                Quasiquote {
                    expr: e1,
                    location: loc1,
                    trailing_newline: tn1,
                },
                Quasiquote {
                    expr: e2,
                    location: loc2,
                    trailing_newline: tn2,
                },
            ) => e1 == e2 && loc1 == loc2 && tn1 == tn2,
            (
                Unquote {
                    expr: e1,
                    location: loc1,
                    trailing_newline: tn1,
                },
                Unquote {
                    expr: e2,
                    location: loc2,
                    trailing_newline: tn2,
                },
            ) => e1 == e2 && loc1 == loc2 && tn1 == tn2,

            (Builtin { name: n1, .. }, Builtin { name: n2, .. }) => n1 == n2,

            (SpecialForm { name: n1, .. }, SpecialForm { name: n2, .. }) => n1 == n2,

            (
                Clausure {
                    args: a1,
                    body: b1,
                    env: e1,
                },
                Clausure {
                    args: a2,
                    body: b2,
                    env: e2,
                },
            ) => a1 == a2 && b1 == b2 && Arc::ptr_eq(e1, e2),

            (
                Macro {
                    args: a1,
                    body: b1,
                    env: e1,
                },
                Macro {
                    args: a2,
                    body: b2,
                    env: e2,
                },
            ) => a1 == a2 && b1 == b2 && Arc::ptr_eq(e1, e2),

            _ => false,
        }
    }
}

impl Expr {
    pub fn symbol(name: &str) -> Self {
        Expr::Symbol {
            name: name.to_string(),
            location: None,
            trailing_newline: false,
        }
    }
    pub fn integer(value: i64) -> Self {
        Expr::Integer {
            value,
            location: None,
            trailing_newline: false,
        }
    }
    pub fn double(value: f64) -> Self {
        Expr::Double {
            value,
            location: None,
            trailing_newline: false,
        }
    }
    pub fn string(value: String) -> Self {
        Expr::String {
            value,
            location: None,
            trailing_newline: false,
        }
    }
    pub fn model(id: ModelId) -> Self {
        Expr::Model {
            id,
            location: None,
            trailing_newline: false,
        }
    }
    pub fn list(elements: Vec<Arc<Expr>>) -> Self {
        Expr::List {
            elements,
            location: None,
            trailing_newline: false,
        }
    }

    #[allow(unused)]
    pub fn nil() -> Self {
        Self::list(vec![])
    }

    #[allow(dead_code)]
    pub fn quote(expr: Expr) -> Self {
        Expr::Quote {
            expr: Box::new(expr),
            location: None,
            trailing_newline: false,
        }
    }

    #[allow(dead_code)]
    pub fn quasiquote(expr: Expr) -> Self {
        Expr::Quasiquote {
            expr: Box::new(expr),
            location: None,
            trailing_newline: false,
        }
    }

    #[allow(dead_code)]
    pub fn unquote(expr: Expr) -> Self {
        Expr::Unquote {
            expr: Box::new(expr),
            location: None,
            trailing_newline: false,
        }
    }
    pub fn is_symbol(&self, name: &str) -> bool {
        match self {
            Expr::Symbol { name: n, .. } => n == name,
            _ => false,
        }
    }
    pub fn as_symbol(&self) -> Result<&str, String> {
        match self {
            Expr::Symbol { name, .. } => Ok(name),
            _ => Err("Not a symbol".to_string()),
        }
    }

    pub fn set_newline(self: Self, b: bool) -> Self {
        match self {
            Expr::Symbol { name, location, .. } => Expr::Symbol {
                name,
                location,
                trailing_newline: b,
            },
            Expr::List {
                elements, location, ..
            } => Expr::List {
                elements,
                location,
                trailing_newline: b,
            },
            Expr::Integer {
                value, location, ..
            } => Expr::Integer {
                value,
                location,
                trailing_newline: b,
            },
            Expr::Double {
                value, location, ..
            } => Expr::Double {
                value,
                location,
                trailing_newline: b,
            },
            Expr::String {
                value, location, ..
            } => Expr::String {
                value,
                location,
                trailing_newline: b,
            },
            Expr::Model { id, location, .. } => Expr::Model {
                id,
                location,
                trailing_newline: b,
            },
            Expr::Quote { expr, location, .. } => Expr::Quote {
                expr,
                location,
                trailing_newline: b,
            },
            Expr::Quasiquote { expr, location, .. } => Expr::Quasiquote {
                expr,
                location,
                trailing_newline: b,
            },
            Expr::Unquote { expr, location, .. } => Expr::Unquote {
                expr,
                location,
                trailing_newline: b,
            },
            Expr::Builtin { .. } => self,
            Expr::SpecialForm { .. } => self,
            Expr::Clausure { .. } => self,
            Expr::Macro { .. } => self,
        }
    }
    pub fn has_newline(&self) -> bool {
        match self {
            Expr::Symbol {
                trailing_newline, ..
            } => *trailing_newline,
            Expr::List {
                trailing_newline, ..
            } => *trailing_newline,
            Expr::Integer {
                trailing_newline, ..
            } => *trailing_newline,
            Expr::Double {
                trailing_newline, ..
            } => *trailing_newline,
            Expr::String {
                trailing_newline, ..
            } => *trailing_newline,
            Expr::Model {
                trailing_newline, ..
            } => *trailing_newline,
            Expr::Quote {
                trailing_newline, ..
            } => *trailing_newline,
            Expr::Quasiquote {
                trailing_newline, ..
            } => *trailing_newline,
            Expr::Unquote {
                trailing_newline, ..
            } => *trailing_newline,
            Expr::Builtin { .. } => false,
            Expr::SpecialForm { .. } => false,
            Expr::Clausure { .. } => false,
            Expr::Macro { .. } => false,
        }
    }
    pub fn location(&self) -> Option<usize> {
        match self {
            Expr::Symbol { location, .. } => *location,
            Expr::List { location, .. } => *location,
            Expr::Integer { location, .. } => *location,
            Expr::Double { location, .. } => *location,
            Expr::String { location, .. } => *location,
            Expr::Model { location, .. } => *location,
            Expr::Quote { location, .. } => *location,
            Expr::Quasiquote { location, .. } => *location,
            Expr::Unquote { location, .. } => *location,
            Expr::Builtin { .. } => None,
            Expr::SpecialForm { .. } => None,
            Expr::Clausure { .. } => None,
            Expr::Macro { .. } => None,
        }
    }
    #[allow(unused)]
    pub fn format(&self) -> String {
        match self {
            Expr::Symbol { name, .. } => name.clone(),
            Expr::List { elements, .. } => {
                let mut s = "(".to_string();
                for (i, e) in elements.iter().enumerate() {
                    s.push_str(&e.format());
                    if i < elements.len() - 1 {
                        s.push(' ');
                    }
                }
                s.push(')');
                s
            }
            Expr::Integer { value, .. } => value.to_string(),
            Expr::Double { value, .. } => value.to_string(),
            Expr::String { value, .. } => format!("\"{}\"", value),
            Expr::Model { location, .. } => {
                format!("<stl mesh at {}>", location.unwrap_or_default())
            }
            Expr::Quote { expr, .. } => format!("'{}", expr.format()),
            Expr::Quasiquote { expr, .. } => format!("`{}", expr.format()),
            Expr::Unquote { expr, .. } => format!("~{}", expr.format()),
            Expr::Builtin { name, .. } => format!("<builtin {}>", name),
            Expr::SpecialForm { name, .. } => format!("<special form {}>", name),
            Expr::Clausure { args, body, .. } => {
                let mut s = "(lambda (".to_string();
                for (i, arg) in args.iter().enumerate() {
                    s.push_str(arg);
                    if i < args.len() - 1 {
                        s.push(' ');
                    }
                }
                s.push_str(") ");
                s.push_str(&body.format());
                s.push(')');
                s
            }
            Expr::Macro { args, body, .. } => {
                let mut s = "(macro (".to_string();
                for (i, arg) in args.iter().enumerate() {
                    s.push_str(arg);
                    if i < args.len() - 1 {
                        s.push(' ');
                    }
                }
                s.push_str(") ");
                s.push_str(&body.format());
                s.push(')');
                s
            }
        }
    }
}

pub fn parse_file(input: &str) -> Result<Vec<Expr>, String> {
    match tokenize(LocatedSpan::new(input)) {
        Ok((_, tokens)) => {
            let mut exprs = vec![];
            let mut rest = &tokens[..];
            while rest.len() > 0 {
                match expr(rest) {
                    Ok((new_rest, expr)) => {
                        exprs.push(expr);
                        rest = new_rest;
                    }
                    Err(e) => return Err(format!("Error: {:?}", e)),
                }
            }
            Ok(exprs)
        }
        Err(e) => Err(format!("Error: {:?}", e)),
    }
}

#[allow(dead_code)]
pub fn parse_expr(input: &str) -> Result<Expr, String> {
    match tokenize(LocatedSpan::new(input)) {
        Ok((_, tokens)) => match expr(&tokens) {
            Ok((_, expr)) => Ok(expr),
            Err(e) => Err(format!("Error: {:?}", e)),
        },
        Err(e) => Err(format!("Error: {:?}", e)),
    }
}

pub type Span<'a> = LocatedSpan<&'a str>;

#[derive(Debug, PartialEq, Clone)]
pub enum Token<'a> {
    Symbol(Span<'a>),
    Integer(Span<'a>),
    Double(Span<'a>),
    Quote(Span<'a>),
    Quasiquote(Span<'a>),
    Unquote(Span<'a>),
    String(Span<'a>),
    LParen(Span<'a>),
    RParen(Span<'a>),
    Newline(Span<'a>),
    Comment(Span<'a>),
}

fn symbol(input: Span) -> IResult<Span, Token> {
    map(
        take_while1(|c: char| c.is_alphanumeric() || "_+-*/<>#?!.".contains(c)),
        Token::Symbol,
    )(input)
}

fn integer(input: Span) -> IResult<Span, Token> {
    map(
        recognize(pair(opt(char('-')), take_while1(|c: char| c.is_digit(10)))),
        |span: Span| Token::Integer(span),
    )(input)
}

fn double(input: Span) -> IResult<Span, Token> {
    map(
        recognize(pair(
            opt(char('-')),
            pair(
                take_while1(|c: char| c.is_digit(10)),
                preceded(char('.'), take_while1(|c: char| c.is_digit(10))),
            ),
        )),
        |span: Span| Token::Double(span),
    )(input)
}

fn string(input: Span) -> IResult<Span, Token> {
    map(
        delimited(char('"'), take_while1(|c: char| c != '"'), char('"')),
        Token::String,
    )(input)
}

fn quote(input: Span) -> IResult<Span, Token> {
    map(char('\''), |_| Token::Quote(input))(input)
}

fn quasiquote(input: Span) -> IResult<Span, Token> {
    map(char('`'), |_| Token::Quasiquote(input))(input)
}

fn unquote(input: Span) -> IResult<Span, Token> {
    map(char('~'), |_| Token::Unquote(input))(input)
}

fn lparen(input: Span) -> IResult<Span, Token> {
    map(char('('), |_| Token::LParen(input))(input)
}

fn rparen(input: Span) -> IResult<Span, Token> {
    map(char(')'), |_| Token::RParen(input))(input)
}

fn newline(input: Span) -> IResult<Span, Token> {
    map(char('\n'), |_| Token::Newline(input))(input)
}

fn comment(input: Span) -> IResult<Span, Token> {
    // A comment starts with a semicolon and consumes everything until a newline or end of input
    let (input, _) = char(';')(input)?;
    let (input, content) = take_while(|c| c != '\n')(input)?;
    Ok((input, Token::Comment(content)))
}

fn tokenize(input: Span) -> IResult<Span, Vec<Token>> {
    // First, collect all tokens including comments
    let (input, all_tokens) = many0(delimited(
        space0,
        alt((
            string, double, integer, symbol, quote, quasiquote, unquote, lparen, rparen, newline,
            comment,
        )),
        space0,
    ))(input)?;

    // Then filter out comments
    let tokens = all_tokens
        .into_iter()
        .filter(|token| !matches!(token, Token::Comment(_)))
        .collect();

    Ok((input, tokens))
}
#[cfg(test)]
mod tokenize_tests {
    use super::*;
    #[test]
    fn test_newline() {
        let result = tokenize(Span::new("\n")).unwrap().1;
        assert_eq!(result, (vec![Token::Newline(Span::new("\n"))]));
    }

    #[test]
    fn test_comment() {
        // Comments should be removed from the token stream
        let input = Span::new("; This is a comment\n(+ 1 2)");
        let result = tokenize(input).unwrap().1;

        // The comment should be stripped, leaving only the expression tokens
        assert_eq!(result.len(), 6); // Newline, LParen, Symbol(+), Integer(1), Integer(2), RParen
        assert!(matches!(result[1], Token::LParen(_)));

        // Test inline comments
        let input = Span::new("(+ 1 2) ; This is an inline comment\n(- 3 4)");
        let result = tokenize(input).unwrap().1;

        // Should have tokens for both expressions, but no comment
        assert_eq!(result.len(), 11); // 5 for first expr, newline, 5 for second expr
    }
}

fn expr<'a>(tokens: &'a [Token]) -> IResult<&'a [Token<'a>], Expr> {
    tuple((
        alt((
            parse_string,
            parse_double,
            parse_integer,
            parse_symbol,
            parse_quote,
            parse_quasiquote,
            parse_unquote,
            parse_list,
        )),
        many0(parse_newline),
    ))(tokens)
    .map(|(input, (expr, newlines))| {
        if newlines.len() > 0 {
            (input, expr.set_newline(true))
        } else {
            (input, expr)
        }
    })
}

fn parse_symbol<'a>(input: &'a [Token]) -> IResult<&'a [Token<'a>], Expr> {
    if let Some((Token::Symbol(span), rest)) = input.split_first() {
        Ok((
            rest,
            Expr::Symbol {
                name: span.fragment().to_string(),
                location: Some(span.location_offset()),
                trailing_newline: false,
            },
        ))
    } else {
        Err(nom::Err::Error(ne::Error::new(input, ErrorKind::Tag)))
    }
}

fn parse_integer<'a>(input: &'a [Token]) -> IResult<&'a [Token<'a>], Expr> {
    if let Some((Token::Integer(span), rest)) = input.split_first() {
        Ok((
            rest,
            Expr::Integer {
                value: span.fragment().parse().unwrap(),
                location: Some(span.location_offset()),
                trailing_newline: false,
            },
        ))
    } else {
        Err(nom::Err::Error(ne::Error::new(input, ErrorKind::Tag)))
    }
}

fn parse_double<'a>(input: &'a [Token]) -> IResult<&'a [Token<'a>], Expr> {
    if let Some((Token::Double(span), rest)) = input.split_first() {
        Ok((
            rest,
            Expr::Double {
                value: span.fragment().parse().unwrap(),
                location: Some(span.location_offset()),
                trailing_newline: false,
            },
        ))
    } else {
        Err(nom::Err::Error(ne::Error::new(input, ErrorKind::Tag)))
    }
}

fn parse_string<'a>(input: &'a [Token]) -> IResult<&'a [Token<'a>], Expr> {
    if let Some((Token::String(span), rest)) = input.split_first() {
        Ok((
            rest,
            Expr::String {
                value: span.fragment().to_string(),
                location: Some(span.location_offset()),
                trailing_newline: false,
            },
        ))
    } else {
        Err(nom::Err::Error(ne::Error::new(input, ErrorKind::Tag)))
    }
}

fn parse_quote<'a>(input: &'a [Token]) -> IResult<&'a [Token<'a>], Expr> {
    if let Some((Token::Quote(span), rest)) = input.split_first() {
        match expr(rest) {
            Ok((rest, expr)) => Ok((
                rest,
                Expr::Quote {
                    expr: Box::new(expr),
                    location: Some(span.location_offset()),
                    trailing_newline: false,
                },
            )),
            Err(e) => Err(e),
        }
    } else {
        Err(nom::Err::Error(ne::Error::new(input, ErrorKind::Tag)))
    }
}

fn parse_quasiquote<'a>(input: &'a [Token]) -> IResult<&'a [Token<'a>], Expr> {
    if let Some((Token::Quasiquote(span), rest)) = input.split_first() {
        match expr(rest) {
            Ok((rest, expr)) => Ok((
                rest,
                Expr::Quasiquote {
                    expr: Box::new(expr),
                    location: Some(span.location_offset()),
                    trailing_newline: false,
                },
            )),
            Err(e) => Err(e),
        }
    } else {
        Err(nom::Err::Error(ne::Error::new(input, ErrorKind::Tag)))
    }
}

fn parse_unquote<'a>(input: &'a [Token]) -> IResult<&'a [Token<'a>], Expr> {
    if let Some((Token::Unquote(span), rest)) = input.split_first() {
        match expr(rest) {
            Ok((rest, expr)) => Ok((
                rest,
                Expr::Unquote {
                    expr: Box::new(expr),
                    location: Some(span.location_offset()),
                    trailing_newline: false,
                },
            )),
            Err(e) => Err(e),
        }
    } else {
        Err(nom::Err::Error(ne::Error::new(input, ErrorKind::Tag)))
    }
}

fn parse_list<'a>(input: &'a [Token]) -> IResult<&'a [Token<'a>], Expr> {
    if let Some((Token::LParen(span), rest)) = input.split_first() {
        let mut elements = vec![];
        let mut rest = rest;
        while let Ok((new_rest, expr)) = expr(rest) {
            elements.push(Arc::new(expr));
            rest = new_rest;
        }
        if let Some((Token::RParen(_), rest)) = rest.split_first() {
            Ok((
                rest,
                Expr::List {
                    elements,
                    location: Some(span.location_offset()),
                    trailing_newline: false,
                },
            ))
        } else {
            Err(nom::Err::Error(ne::Error::new(input, ErrorKind::Tag)))
        }
    } else {
        Err(nom::Err::Error(ne::Error::new(input, ErrorKind::Tag)))
    }
}

fn parse_newline<'a>(input: &'a [Token]) -> IResult<&'a [Token<'a>], Token<'a>> {
    if let Some((Token::Newline(span), rest)) = input.split_first() {
        Ok((rest, Token::Newline(*span)))
    } else {
        Err(nom::Err::Error(ne::Error::new(input, ErrorKind::Tag)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_symbol() {
        let result = parse_expr("hello\n");
        assert_eq!(
            result,
            Ok(Expr::Symbol {
                name: "hello".to_string(),
                location: Some(0),
                trailing_newline: true,
            })
        );
    }

    #[test]
    fn test_integer() {
        let result = parse_expr("123\n");
        assert_eq!(
            result,
            Ok(Expr::Integer {
                value: 123,
                location: Some(0),
                trailing_newline: true,
            })
        );
    }

    #[test]
    fn test_boolean() {
        let result = parse_expr("#t\n");
        assert_eq!(
            result,
            Ok(Expr::Symbol {
                name: "#t".to_string(),
                location: Some(0),
                trailing_newline: true,
            })
        );
    }
    #[test]
    fn test_string1() {
        let result = parse_expr("\"hello\"\n");
        assert_eq!(
            result,
            Ok(Expr::String {
                value: "hello".to_string(),
                location: Some(1),
                trailing_newline: true,
            })
        );
    }
    // [LParen(LocatedSpan { offset: 0, line: 1, fragment: \"(load_expr \\\"hello\\\")\\n\", extra: () }),
    // Symbol(LocatedSpan { offset: 1, line: 1, fragment: \"load\", extra: () })]

    #[test]
    fn test_string2() {
        let result = parse_expr("(load_expr \"hello\")\n");
        assert_eq!(
            result,
            Ok(Expr::List {
                elements: vec![
                    Arc::new(Expr::Symbol {
                        name: "load_expr".to_string(),
                        location: Some(1),
                        trailing_newline: false,
                    }),
                    Arc::new(Expr::String {
                        value: "hello".to_string(),
                        location: Some(12),
                        trailing_newline: false,
                    }),
                ],
                location: Some(0),
                trailing_newline: true,
            })
        );
    }
    #[test]
    fn test_double() {
        let result = parse_expr("123.456\n");
        assert_eq!(
            result,
            Ok(Expr::Double {
                value: 123.456,
                location: Some(0),
                trailing_newline: true,
            })
        );
    }

    #[test]
    fn test_expr() {
        let result = parse_expr("(+ 1 2)\n");
        assert_eq!(
            result,
            Ok(Expr::List {
                elements: vec![
                    Expr::Symbol {
                        name: "+".to_string(),
                        location: Some(1),
                        trailing_newline: false,
                    },
                    Expr::Integer {
                        value: 1,
                        location: Some(3),
                        trailing_newline: false,
                    },
                    Expr::Integer {
                        value: 2,
                        location: Some(5),
                        trailing_newline: false,
                    },
                ]
                .into_iter()
                .map(Arc::new)
                .collect(),
                location: Some(0),
                trailing_newline: true,
            })
        );
    }

    #[test]
    fn test_quote() {
        let result = parse_expr("'(1 2 3)\n");
        assert_eq!(
            result,
            Ok(Expr::Quote {
                expr: Box::new(Expr::List {
                    elements: vec![
                        Expr::Integer {
                            value: 1,
                            location: Some(2),
                            trailing_newline: false,
                        },
                        Expr::Integer {
                            value: 2,
                            location: Some(4),
                            trailing_newline: false,
                        },
                        Expr::Integer {
                            value: 3,
                            location: Some(6),
                            trailing_newline: false,
                        },
                    ]
                    .into_iter()
                    .map(Arc::new)
                    .collect(),

                    location: Some(1),
                    trailing_newline: true,
                }),
                location: Some(0),
                trailing_newline: false,
            })
        );
    }
    #[test]
    fn test_negative_integer() {
        let result = parse_expr("-123\n");
        assert_eq!(
            result,
            Ok(Expr::Integer {
                value: -123,
                location: Some(0),
                trailing_newline: true,
            })
        );
    }

    #[test]
    fn test_multiple_exprs() {
        let result = parse_file("1\n2 3\n");
        assert_eq!(
            result,
            Ok(vec![
                Expr::Integer {
                    value: 1,
                    location: Some(0),
                    trailing_newline: true,
                },
                Expr::Integer {
                    value: 2,
                    location: Some(2),
                    trailing_newline: false,
                },
                Expr::Integer {
                    value: 3,
                    location: Some(4),
                    trailing_newline: true,
                },
            ])
        );
    }
}
