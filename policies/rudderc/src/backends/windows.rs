// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2019-2020 Normation SAS

use std::path::Path;

use anyhow::{bail, Error, Result};
use askama::Template;

use super::Backend;
use crate::ir::{
    condition::Condition,
    technique::{ItemKind, LeafReportingMode, Method, Parameter},
    Technique,
};

pub struct Windows;

impl Default for Windows {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for Windows {
    fn generate(
        &self,
        technique: Technique,
        resources: &Path,
        _standalone: bool,
    ) -> Result<String> {
        // Powershell requires a BOM added at the beginning of all files when using UTF8 encoding
        // See https://docs.microsoft.com/en-us/windows/desktop/intl/using-byte-order-marks
        // Bom for UTF-8 content, three bytes: EF BB BF https://en.wikipedia.org/wiki/Byte_order_mark
        const UTF8_BOM: &[u8; 3] = &[0xef, 0xbb, 0xbf];
        let mut with_bom = String::from_utf8(UTF8_BOM.to_vec()).unwrap();
        with_bom.push_str(&Self::technique(technique, resources)?);
        Ok(with_bom)
    }
}

#[derive(Template)]
#[template(path = "technique.ps1.askama", escape = "none")]
struct TechniqueTemplate<'a> {
    id: &'a str,
    has_modules: bool,
    parameters: Vec<Parameter>,
    methods: Vec<WindowsMethod>,
}

/// Filters for the technique template
pub mod filters {
    use crate::regex;
    use std::fmt::Display;

    fn uppercase_first_letter(s: &str) -> String {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }

    /// `my_method` -> `My-Method`
    pub fn dsc_case<T: Display>(s: T) -> askama::Result<String> {
        Ok(s.to_string()
            .split('_')
            .map(uppercase_first_letter)
            .collect::<Vec<String>>()
            .join("-"))
    }

    /// `my_method` -> `MyMethod`
    pub fn camel_case<T: Display>(s: T) -> askama::Result<String> {
        Ok(s.to_string()
            .split('_')
            .map(uppercase_first_letter)
            .collect::<Vec<String>>()
            .join(""))
    }

    pub fn escape_double_quotes<T: Display>(s: T) -> askama::Result<String> {
        Ok(s.to_string().replace("\"", "`\""))
    }

    pub fn canonify_condition<T: Display>(s: T) -> askama::Result<String> {
        let s = s.to_string();
        Ok(if !s.contains("${") {
            format!("\"{s}\"")
        } else {
            // TODO: does not handle nested vars, we need a parser for this.
            let var = regex!(r"(\$\{[^\}]*})");
            format!(
                "\"{}\"",
                var.replace_all(&s, r##"" + ([Rudder.Condition]::canonify($1)) + ""##)
            )
        })
    }
}

struct WindowsMethod {
    id: String,
    class_prefix: String,
    component_name: String,
    component_key: String,
    disable_reporting: bool,
    condition: Option<String>,
    args: Vec<(String, String)>,
    name: String,
}

impl TryFrom<Method> for WindowsMethod {
    type Error = Error;

    fn try_from(m: Method) -> Result<Self, Self::Error> {
        let Some(report_parameter) = m
            .params
            .get(&m.info.unwrap().class_parameter) else {
            bail!("Missing parameter {}", m.info.unwrap().class_parameter)
        };

        let mut args: Vec<(String, String)> = m.params.clone().into_iter().collect();
        // We want a stable output
        args.sort();

        Ok(Self {
            id: m.id.to_string(),
            class_prefix: m.info.as_ref().unwrap().class_prefix.clone(),
            component_name: m.name,
            component_key: report_parameter.to_string(),
            disable_reporting: m.reporting.mode == LeafReportingMode::Disabled,
            // FIXME: None
            condition: Some(m.condition.to_string()),
            args,
            name: Windows::pascal_case(&m.info.as_ref().unwrap().bundle_name),
        })
    }
}

impl Windows {
    pub fn new() -> Self {
        Self
    }

    fn pascal_case(s: &str) -> String {
        let chars = s.chars();

        let mut pascal = String::new();
        let mut is_next_uppercase = true;
        for c in chars {
            let next = match c {
                ' ' | '_' | '-' => {
                    is_next_uppercase = true;
                    String::new()
                }
                c => {
                    if is_next_uppercase {
                        is_next_uppercase = false;
                        c.to_uppercase().to_string()
                    } else {
                        c.to_string()
                    }
                }
            };
            pascal.push_str(&next);
        }
        pascal
    }

    pub fn technique_name(s: &str) -> String {
        Self::pascal_case(s)
    }

    fn technique(src: Technique, resources: &Path) -> Result<String> {
        // Extract methods
        fn resolve_module(r: ItemKind, context: Condition) -> Result<Vec<WindowsMethod>> {
            match r {
                ItemKind::Block(r) => {
                    let mut calls: Vec<WindowsMethod> = vec![];
                    for inner in r.items {
                        calls.extend(resolve_module(inner, context.and(&r.condition))?);
                    }
                    Ok(calls)
                }
                ItemKind::Method(r) => {
                    let method: Vec<WindowsMethod> = vec![r.try_into()?];
                    Ok(method)
                }
                _ => todo!(),
            }
        }

        let mut methods = vec![];
        for item in src.items {
            for call in resolve_module(item, Condition::Defined)? {
                methods.push(call);
            }
        }

        let technique = TechniqueTemplate {
            id: &src.id.to_string(),
            has_modules: !Windows::list_resources(resources)?.is_empty(),
            parameters: src.parameters,
            methods,
        };
        technique.render().map_err(|e| e.into())
    }
}
