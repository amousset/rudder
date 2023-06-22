// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2023 Normation SAS

//! Rudder expression parser
//!
//! Similar to what the webapp does at generation, but we only use it to lint
//! and/or transform, but not resolve any value.
//!
//! NOTE: All the technique content will ONLY be interpreted by the target platforms, NOT the webapp.
//! We hence only support what the agent support to provide better feedback to the developers.
//! i.e. no `| options`, so `${ spaces . anywhere }`, etc.

//! TODO: add warnings when using instance-specific values (node properties, etc.)

//! TODO: specific parser for condition expressions

use anyhow::Error;
use nom::branch::alt;
use nom::bytes::complete::{take_till};
use nom::combinator::eof;
use nom::multi::{many0, many1};
use nom::{bytes::complete::tag, IResult};
use std::str::FromStr;

// ${node.properties[dns_${sys.host}]}
#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    /// `${sys.host}`
    CfengineSys(Box<Expression>),
    /// `${rudder.param.NAME}` (deprecated)
    /// `${rudder.parameters[NAME]}`
    Parameter(Box<Expression>),
    /// `${rudder.node.NAME}`
    /// `${rudder.node.NAME.SUB_NAME}`
    Node(Box<Expression>),
    /// `${node.property[KEY][SUBKEY]}`
    Property(Vec<Expression>),
    /// `${anything_unidentified}` (includes Rudder engines, etc.)
    OtherVar(Box<Expression>),
    /// A static value
    Scalar(String),
    /// A list of tokens
    Sequence(Vec<Expression>),
}

impl FromStr for Expression {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, res) = expression(s).unwrap();
        Ok(res.clone())
    }
}

fn expression(s: &str) -> IResult<&str, Expression> {
    let (s, out) = many0(alt((
        // different types of tokens
        node_properties,
        generic_var,
        parameter_legacy,
        parameter,
        // default
        string,
    )))(s)?;
    Ok((s, Expression::Sequence(out)))
}

// Reads until beginning or end of variable
fn string(s: &str) -> IResult<&str, Expression> {
    let (s, out) = alt((take_till(|c| [']', '}', '$'].contains(&c)), eof))(s)?;
    Ok((s, Expression::Scalar(out.to_string())))
}

// Reads a node property
fn generic_var(s: &str) -> IResult<&str, Expression> {
    let (s, _) = tag("${")(s)?;
    let (s, out) = expression(s)?;
    let (s, _) = tag("}")(s)?;
    Ok((s, Expression::OtherVar(Box::new(out))))
}

fn parameter_legacy(s: &str) -> IResult<&str, Expression> {
    let (s, _) = tag("${")(s)?;
    let (s, _) = tag("rudder.param.")(s)?;
    let (s, out) = expression(s)?;
    let (s, _) = tag("}")(s)?;
    Ok((s, Expression::Parameter(Box::new(out))))
}

fn parameter(s: &str) -> IResult<&str, Expression> {
    let (s, _) = tag("${")(s)?;
    let (s, _) = tag("rudder.parameters")(s)?;
    // FIXME: list of subkeys?
    let (s, out) = key(s)?;
    Ok((s, Expression::Parameter(Box::new(out))))
}

// Reads a node property
fn node_properties(s: &str) -> IResult<&str, Expression> {
    let (s, _) = tag("${")(s)?;
    let (s, _) = tag("node.properties")(s)?;
    // keys
    let (s, keys) = many1(key)(s)?;
    let (s, _) = tag("}")(s)?;
    Ok((s, Expression::Property(keys)))
}

// Reads a key in square brackets
fn key(s: &str) -> IResult<&str, Expression> {
    let (s, _) = tag("[")(s)?;
    let (s, key) = expression(s)?;
    let (s, _) = tag("]")(s)?;
    Ok((s, key))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_reads_keys() {
        let (_, out) = key("[toto]").unwrap();
        assert_eq!(out, Expression::Scalar("toto".to_string()))
    }

    #[test]
    fn it_reads_node_properties() {
        let (_, out) = node_properties("${node.properties[toto]}").unwrap();
        assert_eq!(
            out,
            Expression::Property(vec![Expression::Scalar("toto".to_string())])
        );
        let (_, out) = node_properties("${node.properties[toto][tutu]}").unwrap();
        assert_eq!(
            out,
            Expression::Property(vec![
                Expression::Scalar("toto".to_string()),
                Expression::Scalar("tutu".to_string())
            ])
        );
        let (_, out) =
            node_properties("${node.properties[${node.properties[inner]}][tutu]}").unwrap();
        assert_eq!(
            out,
            Expression::Property(vec![
                Expression::Scalar("toto".to_string()),
                Expression::Scalar("tutu".to_string())
            ])
        );
    }

    #[test]
    fn it_reads_generic_var() {
        let (_, out) = generic_var("${plouf}").unwrap();
        assert_eq!(
            out,
            Expression::OtherVar(Box::new(Expression::Scalar("plouf".to_string())))
        );
    }

    #[test]
    fn it_reads_parameters() {
        let (_, out) = parameter_legacy("${rudder.param.plouf}").unwrap();
        assert_eq!(
            out,
            Expression::Parameter(Box::new(Expression::Scalar("plouf".to_string())))
        );
        let (_, out) = parameter("${rudder.parameters[plouf]}").unwrap();
        assert_eq!(
            out,
            Expression::Parameter(Box::new(Expression::Scalar("plouf".to_string())))
        );
    }
}
