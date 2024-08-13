use nom::{
    branch::alt,
    bytes::complete::take_while1,
    character::complete::{char, multispace0},
    combinator::map,
    multi::many0,
    sequence::delimited,
    IResult,
};

use nom_locate::LocatedSpan;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Symbol {
        name: String,
        location: usize,
    },
    List {
        elements: Vec<Expr>,
        location: usize,
    },
}

type Span<'a> = LocatedSpan<&'a str>;

fn symbol(input: Span) -> IResult<Span, Expr> {
    map(
        take_while1(|c: char| c.is_alphanumeric() || "+-*/".contains(c)),
        |s: Span| Expr::Symbol {
            name: s.fragment().to_string(),
            location: s.location_offset(),
        },
    )(input)
}

fn list(input: Span) -> IResult<Span, Expr> {
    map(
        delimited(
            char('('),
            many0(delimited(multispace0, expr, multispace0)),
            char(')'),
        ),
        |elements: Vec<Expr>| Expr::List {
            elements,
            location: input.location_offset(),
        },
    )(input)
}

pub fn expr(input: Span) -> IResult<Span, Expr> {
    alt((symbol, list))(input)
}

pub fn main() {
    let input = Span::new("(define (square x) (* x x))");
    let result = expr(input);
    println!("{:?}", result);
}
