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
// #[serde(untagged)]
#[serde(tag = "type")]
pub enum Variable {
    Regex {
        name: String,
        field: String,
        #[serde(deserialize_with = "de_regex")]
        regex: Regex,
    },
    Exact {
        name: String,
        field: String,
        #[serde(deserialize_with = "de_bytes")]
        exact: Vec<u8>,
    },
    Contains {
        name: String,
        field: String,
        #[serde(deserialize_with = "de_bytes")]
        contains: Vec<u8>,
    },
}

fn de_regex<'de, D>(deserializer: D) -> Result<Regex, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    Regex::new(&s).map_err(|e| serde::de::Error::custom(e))
}

fn de_bytes<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    Ok(s.as_bytes().to_owned())
}

impl Variable {
    pub fn get_field(&self) -> &str {
        match self {
            Self::Regex {
                ref field,
                regex: _,
                name: _,
            } => field,
            Self::Contains {
                ref field,
                contains: _,
                name: _,
            } => field,
            Self::Exact {
                ref field,
                exact: _,
                name: _,
            } => field,
        }
    }

    pub fn match_against(&self, json: &str) -> (&str, bool) {
        match &self {
            Variable::Regex {
                ref name,
                field,
                regex,
            } => (name, regex.is_match(gjson::get(json, field).str())),
            Variable::Exact {
                ref name,
                field,
                exact,
            } => (name, gjson::get(json, field).str().as_bytes() == exact),
            Variable::Contains {
                ref name,
                field,
                ref contains,
            } => {
                if contains.len() > gjson::get(json, field).str().as_bytes().len() {
                    return (name, false);
                } else if contains.len() == gjson::get(json, field).str().as_bytes().len() {
                    return (name, gjson::get(json, field).str().as_bytes() == contains);
                }
                //
                let increment = gjson::get(json, field).str().as_bytes().len()
                    - gjson::get(json, field).str().as_bytes().len() % contains.len();
                let data: Vec<u8> = gjson::get(json, field).str().as_bytes().to_owned();
                for i in 0..(data.len() % contains.len() + 1) {
                    // println!("data: {:?} slice: {:?}", data, slice);
                    // println!("range: {:?}", (i)..(i + increment));
                    // println!("conditional: {:?}", (slice == contains));
                    // is_match = is_match | (slice == contains);
                    if &data[i..i + increment] == contains {
                        return (name, true);
                    }
                }
                (name, false)
            }
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
                Variable::Exact {
                    name,
                    field: _,
                    exact: _,
                } => name.as_str(),
                Variable::Contains {
                    name,
                    field: _,
                    contains: _,
                } => name.as_str(),
                Variable::Regex {
                    name,
                    field: _,
                    regex: _,
                } => name.as_str(),
            })
            .collect();
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

    pub fn get_matches_json(&self, json: &str) -> HashMap<String, String> {
        self.variables
            .iter()
            .map(|v| {
                (
                    v.get_field().into(),
                    gjson::get(json, v.get_field()).str().into(),
                )
            })
            .collect()
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
                Variable::Contains {
                    name: "A".to_string(),
                    field: "field".to_string(),
                    contains: "abc123".as_bytes().to_vec(),
                },
                Variable::Regex {
                    name: "B".to_string(),
                    field: "obj*.field".to_string(),
                    regex: Regex::new("[a-z]{3}[0-9]{3}").unwrap(),
                },
            ],
            conditional: "A and B".to_string(),
        };
        let json = r#"{ "field": "xabc1234", "object": { "field": "xyz321" } }"#;
        rule = rule.validate().unwrap();
        assert!(rule.match_json(json));
    }
}
