#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate pest;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde;
extern crate regex;

mod conditionals;
use regex::Regex;
use std::{collections::HashMap, error::Error};

#[derive(Deserialize)]
pub struct Rule {
    pub name: String,
    pub variables: Vec<Variable>,
    pub conditional: String,
}

#[derive(Deserialize)]
pub enum Variable {
    Regex {
        name: String,
        field: String,
        #[serde(deserialize_with = "de_regex")]
        regex: Regex,
    },
    Raw {
        name: String,
        field: String,
        data: Vec<u8>,
    },
}

fn de_regex<'de, D>(deserializer: D) -> Result<Regex, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = serde::Deserialize::deserialize(deserializer)?;
    Regex::new(s).map_err(|e| serde::de::Error::custom(e))
}

impl Variable {
    pub fn match_against(&self, json: &str) -> (&str, bool) {
        match &self {
            Variable::Regex {
                ref name,
                field,
                regex,
            } => {
                println!("Data: {}", gjson::get(json, field).str());
                (name, regex.is_match(gjson::get(json, field).str()))
            }
            Variable::Raw {
                ref name,
                field,
                data,
            } => (name, gjson::get(json, field).str().as_bytes() == data),
        }
    }
}

#[derive(Serialize)]
pub struct Conditional {
    pub raw: String,
    // #[serde(skip)]
    // inner: conditionals::ConditionalInner<'a>,
}

impl Conditional {
    /// Creates an, unvalidated, conditional.
    pub fn new<'a>(conditional: &str) -> Result<Self, conditionals::ParseError> {
        // let inner: conditionals::ConditionalInner = conditionals::parse(conditional)?;
        Ok(Self {
            raw: conditional.into(),
            // inner,
        })
    }
    pub fn eval(&self, variables: &HashMap<&str, bool>) -> bool {
        conditionals::eval(conditionals::parse(&self.raw).unwrap(), variables)
    }
    pub fn validate(self, variables: Vec<&str>) -> bool {
        conditionals::validate(conditionals::parse(&self.raw).unwrap(), variables)
    }
}

//
impl Rule {
    pub fn validate(self) -> Result<Self, Box<dyn Error>> {
        let keys = self
            .variables
            .iter()
            .map(|v| match &v {
                Variable::Raw {
                    name,
                    field: _,
                    data: _,
                } => name.as_str(),
                Variable::Regex {
                    name,
                    field: _,
                    regex: _,
                } => name.as_str(),
            })
            .collect();
        println!("{:?}", keys);
        match Conditional::new(&self.conditional)?.validate(keys) {
            true => Ok(self),
            false => Err("Could not validate conditional statement".into()),
        }
    }

    pub fn match_json(&self, json: &str) -> bool {
        let mut map = HashMap::new();
        // Collect and match each Variable
        for v in &self.variables {
            let (k, v) = v.match_against(json);
            map.insert(k, v);
        }
        // Run results map against Conditional
        Conditional::new(&self.conditional).unwrap().eval(&map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rule() {
        let mut rule = Rule {
            name: "test_rule".to_string(),
            variables: vec![
                Variable::Raw {
                    name: "A".to_string(),
                    field: "field".to_string(),
                    data: "abc123".as_bytes().to_vec(),
                },
                Variable::Regex {
                    name: "B".to_string(),
                    field: "obj*.field".to_string(),
                    regex: Regex::new("[a-z]{3}[0-9]{3}").unwrap(),
                },
            ],
            conditional: "A and B".to_string(),
        };

        let json = r#"{ "field": "abc123", "object": { "field": "xyz321" } }"#;
        //
        rule = rule.validate().unwrap();

        assert!(rule.match_json(json));
    }
}
