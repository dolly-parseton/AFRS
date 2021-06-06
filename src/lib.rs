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
use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use std::{cmp, collections::HashMap, error::Error, fmt};

// #[derive(Deserialize)]
pub struct Rule {
    pub name: String,
    pub variables: Vec<Variable>,
    // #[serde(deserialize_with = "de_conditional")]
    pub conditional: Conditional,
}

impl<'de> Deserialize<'de> for Rule {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Name,
            Variables,
            Conditional,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`name` or `variables` or `conditional`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "name" => Ok(Field::Name),
                            "variables" => Ok(Field::Variables),
                            "conditional" => Ok(Field::Conditional),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }
        struct RuleVisitor;
        impl<'de> Visitor<'de> for RuleVisitor {
            type Value = Rule;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Rule")
            }
            fn visit_map<V>(self, mut map: V) -> Result<Rule, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut name: Option<String> = None;
                let mut variables: Option<Vec<Variable>> = None;
                let mut raw_conditional: Option<String> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Name => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        Field::Variables => {
                            if variables.is_some() {
                                return Err(de::Error::duplicate_field("variables"));
                            }
                            variables = Some(map.next_value()?);
                        }
                        Field::Conditional => {
                            if raw_conditional.is_some() {
                                return Err(de::Error::duplicate_field("conditional"));
                            }
                            raw_conditional = Some(map.next_value()?);
                        }
                    }
                }
                let conditional = match (&variables, &raw_conditional) {
                    (Some(variables), Some(conditional)) => {
                        //
                        match Conditional::new(
                            &conditional,
                            variables.iter().map(|v| v.get_name().to_string()).collect(),
                        ) {
                            Ok(c) => Ok(c),
                            Err(e) => Err(serde::de::Error::custom(e)),
                        }
                    }
                    _ => return Err(de::Error::missing_field("conditional")),
                }?;
                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                let variables = variables.ok_or_else(|| de::Error::missing_field("variables"))?;
                Ok(Rule {
                    name,
                    variables,
                    conditional,
                })
            }
        }

        const FIELDS: &[&str] = &["name", "variables", "conditional"];
        deserializer.deserialize_struct("Rule", FIELDS, RuleVisitor)
    }
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
    match Regex::new(&s) {
        Ok(r) => Ok(r),
        Err(e) => Err(serde::de::Error::custom(e)),
    }
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
    pub fn get_name(&self) -> &str {
        match self {
            Self::Regex {
                field: _,
                regex: _,
                ref name,
            } => name,
            Self::Contains {
                field: _,
                contains: _,
                ref name,
            } => name,
            Self::Exact {
                field: _,
                exact: _,
                ref name,
            } => name,
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
                match contains
                    .len()
                    .cmp(&gjson::get(json, field).str().as_bytes().len())
                {
                    cmp::Ordering::Greater => (name, false),
                    cmp::Ordering::Equal => {
                        (name, gjson::get(json, field).str().as_bytes() == contains)
                    }
                    cmp::Ordering::Less => {
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
    }
}

pub struct Conditional {
    pub raw: String,
    pub variables: Vec<String>,
}

impl Conditional {
    /// Creates an, unvalidated, conditional.
    pub fn new(
        conditional: &str,
        variables: Vec<String>,
    ) -> Result<Self, conditionals::ParseError> {
        let inner = conditionals::parse(conditional)?;
        if !conditionals::validate(inner, variables.iter().map(|v| v.as_str()).collect()) {
            return Err(conditionals::ParseError {
                reason: "Could not validate conditional statement".into(),
            });
        }
        Ok(Self {
            raw: conditional.into(),
            variables,
            // eval: Box::new(eval),
        })
    }
    pub fn eval(&self, variables: &HashMap<&str, bool>) -> bool {
        conditionals::eval(conditionals::parse(&self.raw).unwrap(), variables)
    }
    pub fn validate(&self, variables: Vec<&str>) -> bool {
        conditionals::validate(conditionals::parse(&self.raw).unwrap(), variables)
    }
}

//
impl Rule {
    pub fn validate(self) -> Result<Self, Box<dyn Error>> {
        let keys = self.variables.iter().map(|v| v.get_name()).collect();
        match self.conditional.validate(keys) {
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
        self.conditional.eval(&map)
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
            conditional: Conditional::new("A and B", vec!["A".to_string(), "B".to_string()])
                .unwrap(),
        };
        let json = r#"{ "field": "xabc1234", "object": { "field": "xyz321" } }"#;
        rule = rule.validate().unwrap();
        assert!(rule.match_json(json));
    }
}
