use std::collections::HashMap;

extern crate chrono;
use chrono::{DateTime, Utc};

#[macro_use]
extern crate lazy_static;

extern crate regex;
use regex::Regex;

lazy_static! {
    static ref HELP_RE: Regex = Regex::new(r"^#\s+HELP\s+(.+)$").unwrap();
    static ref TYPE_RE: Regex = Regex::new(r"^#\s+TYPE\s+(\w+)\s+(\w+)").unwrap();
    static ref SAMPLE_RE: Regex = Regex::new(
        r"^(?P<name>\w+)(\{(?P<labels>[^}]+)\})?\s+(?P<value>\w+)(\s+(?P<timestamp>\d+))?"
    )
    .unwrap();
}

#[derive(Debug, Eq, PartialEq)]
pub enum LineInfo<'a> {
    Docstring(&'a str),
    Type {
        metric_name: &'a str,
        type_name: &'a str,
    },
    Sample {
        metric_name: &'a str,
        labels: Option<&'a str>,
        value: &'a str,
        timestamp: Option<&'a str>,
    },
    Empty,
    Ignored,
}

impl<'a> LineInfo<'a> {
    pub fn parse(line: &'a str) -> LineInfo<'a> {
        let line = line.trim();
        if line.len() == 0 {
            return LineInfo::Empty;
        }
        match HELP_RE.captures(line) {
            Some(ref caps) => {
                return match caps.get(1) {
                    Some(ref s) => LineInfo::Docstring(s.as_str()),
                    _ => LineInfo::Ignored,
                }
            }
            None => {}
        }
        match TYPE_RE.captures(line) {
            Some(ref caps) => {
                return match (caps.get(1), caps.get(2)) {
                    (Some(ref metric_name), Some(ref type_name)) => LineInfo::Type {
                        metric_name: metric_name.as_str(),
                        type_name: type_name.as_str(),
                    },
                    _ => LineInfo::Ignored,
                }
            }
            None => {}
        }
        match SAMPLE_RE.captures(line) {
            Some(ref caps) => {
                return match (
                    caps.name("name"),
                    caps.name("labels"),
                    caps.name("value"),
                    caps.name("timestamp"),
                ) {
                    (Some(ref name), labels, Some(ref value), timestamp) => LineInfo::Sample {
                        metric_name: name.as_str(),
                        labels: labels.map_or(None, |c| Some(c.as_str())),
                        value: value.as_str(),
                        timestamp: timestamp.map_or(None, |c| Some(c.as_str())),
                    },
                    _ => LineInfo::Ignored,
                }
            }
            None => LineInfo::Ignored,
        }
    }
}

pub struct Sample {
    metric: String,
    value: Value,
    labels: Labels,
    timestamp: DateTime<Utc>,
}

pub struct SamplesLessThan(f64, f64);

pub struct SamplesInQuantile(f64, f64);

pub type Labels = HashMap<String, String>;

pub enum Value {
    Counter(f64),
    Gauge(f64),
    Histogram(Vec<SamplesLessThan>),
    Summary(Vec<SamplesInQuantile>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lineinfo_parse() {
        assert_eq!(
            LineInfo::parse("foo 2"),
            LineInfo::Sample {
                metric_name: "foo",
                value: "2",
                labels: None,
                timestamp: None,
            }
        );
        assert_eq!(LineInfo::parse("foo=2"), LineInfo::Ignored,);
        assert_eq!(
            LineInfo::parse("foo 2 1543182234"),
            LineInfo::Sample {
                metric_name: "foo",
                value: "2",
                labels: None,
                timestamp: Some("1543182234"),
            }
        );
        assert_eq!(
            LineInfo::parse("foo{bar=baz} 2 1543182234"),
            LineInfo::Sample {
                metric_name: "foo",
                value: "2",
                labels: Some("bar=baz"),
                timestamp: Some("1543182234"),
            }
        );
        assert_eq!(
            LineInfo::parse("foo{bar=baz,quux=nonce} 2 1543182234"),
            LineInfo::Sample {
                metric_name: "foo",
                value: "2",
                labels: Some("bar=baz,quux=nonce"),
                timestamp: Some("1543182234"),
            }
        );
        assert_eq!(
            LineInfo::parse("# HELP this is a docstring"),
            LineInfo::Docstring("this is a docstring"),
        );
        assert_eq!(
            LineInfo::parse("# TYPE foobar bazquux"),
            LineInfo::Type {
                metric_name: "foobar",
                type_name: "bazquux"
            },
        );
    }
}
