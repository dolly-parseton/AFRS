use pest::{
    iterators::{Pair, Pairs},
    prec_climber::{Assoc, Operator, PrecClimber},
    Parser,
};
use std::{collections::HashMap, error::Error, fmt};

/// `ConditionalParser` uses [`Pest`](https://docs.rs/pest/2.1.3/pest/) to parse the conditional string.
#[derive(Parser)]
#[grammar = "conditional.pest"]
pub struct ConditionalParser {}

lazy_static! {
    static ref PREC_CLIMBER: PrecClimber<Rule> = {
        use Assoc::*;
        PrecClimber::new(vec![
            Operator::new(Rule::or, Left),
            Operator::new(Rule::and, Left),
        ])
    };
}

/// Error struct returned when evaluating the conditional.
#[derive(Debug)]
pub struct EvalError {
    pub reason: String,
}

impl Error for EvalError {}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EvalError: {}", self.reason)
    }
}

/// Error struct returned when parsing the conditional.
#[derive(Debug)]
pub struct ParseError {
    pub reason: String,
}

impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ParseError: {}", self.reason)
    }
}

/// Function used when validating a conditional is valid, checks the variable names against what the condtional string contains recursively.
pub fn validate(expression: Pairs<Rule>, variables: Vec<&str>) -> bool {
    fn inner(pairs: Pairs<Rule>, variables: &Vec<&str>, valid: &mut bool) {
        pairs
            .into_iter()
            .map(|pair| match pair.as_rule() {
                Rule::variable => *valid = variables.contains(&pair.as_str()) & *valid,
                Rule::expr => inner(pair.into_inner(), variables, valid),
                _ => (),
            })
            .for_each(drop);
    }
    let mut valid = true;
    inner(expression, &variables, &mut valid);
    valid
}

/// Function checks the conditional against a HashMap of variable names and the boolean each variable resolves to.
pub fn eval(expression: Pairs<Rule>, variables: &HashMap<&str, bool>) -> bool {
    PREC_CLIMBER.climb(
        expression,
        |pair: Pair<Rule>| match pair.as_rule() {
            Rule::variable => variables.get(pair.as_str()).map(|v| *v).unwrap(),
            Rule::expr => eval(pair.into_inner(), variables),
            _ => unreachable!(),
        },
        |lhs: bool, op: Pair<Rule>, rhs: bool| match op.as_rule() {
            Rule::or => lhs | rhs,
            Rule::and => lhs & rhs,
            _ => unreachable!(),
        },
    )
}

pub type ConditionalInner<'a> = Pairs<'a, Rule>;
pub fn parse(s: &str) -> Result<ConditionalInner<'_>, ParseError> {
    ConditionalParser::parse(Rule::conditional, s).map_err(|_| ParseError {
        reason: "Unable to parse conditional".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let mut map = HashMap::new();
        map.insert("A", true);
        map.insert("B", false);
        map.insert("C", false);
        map.insert("D", false);
        let parser = ConditionalParser::parse(Rule::conditional, "A | C").unwrap();

        let keys: Vec<&str> = map.keys().map(|v| *v).collect();
        println!("{:?}", parser);
        println!("Validate: {:?}", validate(parser.clone(), keys));
        println!("Result: {:?}", eval(parser, &map));
    }
}
