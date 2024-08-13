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
    Integer {
        value: i64,
        location: usize,
    },
    Double {
        value: f64,
        location: usize,
    },
}

type Span<'a> = LocatedSpan<&'a str>;

fn symbol(input: Span) -> IResult<Span, Expr> {
    map(
        take_while1(|c: char| c.is_alphanumeric() || "+-*/<>".contains(c)),
        |s: Span| Expr::Symbol {
            name: s.fragment().to_string(),
            location: s.location_offset(),
        },
    )(input)
}

fn integer(input: Span) -> IResult<Span, Expr> {
    map(take_while1(|c: char| c.is_digit(10)), |s: Span| {
        Expr::Integer {
            value: s.fragment().parse().unwrap(),
            location: s.location_offset(),
        }
    })(input)
}
fn double(input: Span) -> IResult<Span, Expr> {
    map(
        take_while1(|c: char| c.is_digit(10) || c == '.'),
        |s: Span| Expr::Double {
            value: s.fragment().parse().unwrap(),
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
    alt((integer, double, symbol, list))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol() {
        let result = symbol(Span::new("hello"));
        assert_eq!(get_symbol_name(result), Some("hello".to_string()));
    }
    fn get_symbol_name(result: IResult<Span, Expr>) -> Option<String> {
        match result.unwrap().1 {
            Expr::Symbol { name, location: _ } => Some(name),
            _ => None,
        }
    }

    #[test]
    fn test_integer() {
        let result = integer(Span::new("123"));
        assert_eq!(get_integer_value(result), Some(123));
    }
    fn get_integer_value(result: IResult<Span, Expr>) -> Option<i64> {
        match result.unwrap().1 {
            Expr::Integer { value, location: _ } => Some(value),
            _ => None,
        }
    }

    #[test]
    fn test_double() {
        let result = double(Span::new("123.456"));
        assert_eq!(get_double_value(result), Some(123.456));
    }
    fn get_double_value(result: IResult<Span, Expr>) -> Option<f64> {
        match result.unwrap().1 {
            Expr::Double { value, location: _ } => Some(value),
            _ => None,
        }
    }
    #[test]
    fn test_expr() {
        let result = expr(Span::new("(+ 1 2)"));
        assert_eq!(
            get_expr(result),
            Some(Expr::List {
                elements: vec![
                    Expr::Symbol {
                        name: "+".to_string(),
                        location: 1,
                    },
                    Expr::Integer {
                        value: 1,
                        location: 3,
                    },
                    Expr::Integer {
                        value: 2,
                        location: 5,
                    },
                ],
                location: 0,
            })
        );
    }
    fn get_expr(result: IResult<Span, Expr>) -> Option<Expr> {
        match result.unwrap().1 {
            Expr::List {
                elements,
                location: _,
            } => Some(Expr::List {
                elements,
                location: 0,
            }),
            _ => None,
        }
    }
}
