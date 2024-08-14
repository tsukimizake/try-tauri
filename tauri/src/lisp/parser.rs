use nom::{
    branch::alt,
    bytes::complete::take_while1,
    character::complete::{char, multispace0},
    combinator::{map, opt},
    multi::many0,
    sequence::{delimited, preceded},
    IResult,
};

use nom_locate::LocatedSpan;

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Symbol {
        name: String,
        location: Option<usize>,
    },
    List {
        elements: Vec<Expr>,
        location: Option<usize>,
    },
    Integer {
        value: i64,
        location: Option<usize>,
    },
    Double {
        value: f64,
        location: Option<usize>,
    },
    QuotedList {
        elements: Vec<Expr>,
        location: Option<usize>,
    },
    Builtin(fn(&[Expr]) -> Result<Expr, String>),
}

impl Expr {
    pub fn symbol(name: &str) -> Self {
        Expr::Symbol {
            name: name.to_string(),
            location: None,
        }
    }
    pub fn integer(value: i64) -> Self {
        Expr::Integer {
            value,
            location: None,
        }
    }
    pub fn double(value: f64) -> Self {
        Expr::Double {
            value,
            location: None,
        }
    }
    pub fn list(elements: Vec<Expr>) -> Self {
        Expr::List {
            elements,
            location: None,
        }
    }
    pub fn quoted_list(elements: Vec<Expr>) -> Self {
        Expr::QuotedList {
            elements,
            location: None,
        }
    }
}

pub fn run(input: &str) -> Result<Expr, String> {
    match expr(LocatedSpan::new(input)) {
        Ok((_, expr)) => Ok(expr),
        Err(e) => Err(format!("Error: {:?}", e)),
    }
}

pub type Span<'a> = LocatedSpan<&'a str>;

fn symbol(input: Span) -> IResult<Span, Expr> {
    map(
        take_while1(|c: char| c.is_alphanumeric() || "+-*/<>".contains(c)),
        |s: Span| Expr::Symbol {
            name: s.fragment().to_string(),
            location: Some(s.location_offset()),
        },
    )(input)
}

fn integer(input: Span) -> IResult<Span, Expr> {
    map(take_while1(|c: char| c.is_digit(10)), |s: Span| {
        Expr::Integer {
            value: s.fragment().parse().unwrap(),
            location: Some(s.location_offset()),
        }
    })(input)
}
fn double(input: Span) -> IResult<Span, Expr> {
    map(
        take_while1(|c: char| c.is_digit(10) || c == '.'),
        |s: Span| Expr::Double {
            value: s.fragment().parse().unwrap(),
            location: Some(s.location_offset()),
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
            location: Some(input.location_offset()),
        },
    )(input)
}

fn quoted_list(input: Span) -> IResult<Span, Expr> {
    map(
        preceded(char('\''), list),
        |list_expr: Expr| match list_expr {
            Expr::List { elements, location } => Expr::QuotedList { elements, location },
            _ => unreachable!(),
        },
    )(input)
}
fn expr(input: Span) -> IResult<Span, Expr> {
    alt((integer, double, symbol, quoted_list, list))(input)
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
                        location: Some(1),
                    },
                    Expr::Integer {
                        value: 1,
                        location: Some(3),
                    },
                    Expr::Integer {
                        value: 2,
                        location: Some(5),
                    },
                ],
                location: Some(0),
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
                location: Some(0),
            }),
            _ => None,
        }
    }

    #[test]
    fn test_quoted_list() {
        let result = expr(Span::new("'(1 2 3)"));
        assert_eq!(
            get_quoted_list(result),
            Some(Expr::QuotedList {
                elements: vec![
                    Expr::Integer {
                        value: 1,
                        location: Some(2),
                    },
                    Expr::Integer {
                        value: 2,
                        location: Some(4),
                    },
                    Expr::Integer {
                        value: 3,
                        location: Some(6),
                    },
                ],
                location: Some(0),
            })
        );
    }
    fn get_quoted_list(result: IResult<Span, Expr>) -> Option<Expr> {
        match result.unwrap().1 {
            Expr::QuotedList {
                elements,
                location: _,
            } => Some(Expr::QuotedList {
                elements,
                location: Some(0),
            }),
            _ => None,
        }
    }
}
