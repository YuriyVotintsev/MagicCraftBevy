use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct CalcTemplateRaw {
    pub name: String,
    pub params: Vec<String>,
    pub formula: String,
}

#[derive(Debug, Clone)]
struct CalcTemplate {
    params: Vec<String>,
    formula: String,
}

#[derive(Debug, Clone, Default, Resource)]
pub struct CalcRegistry {
    templates: HashMap<String, CalcTemplate>,
}

impl CalcRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: String, params: Vec<String>, formula: String) {
        self.templates.insert(name, CalcTemplate { params, formula });
    }

    pub fn from_raw(raws: &[CalcTemplateRaw]) -> Self {
        let mut registry = Self::new();
        for raw in raws {
            registry.add(raw.name.clone(), raw.params.clone(), raw.formula.clone());
        }
        registry
    }

    pub fn expand(&self, input: &str) -> Result<String, String> {
        self.expand_recursive(input, 0)
    }

    fn expand_recursive(&self, input: &str, depth: usize) -> Result<String, String> {
        if depth > 16 {
            return Err(format!("calc() expansion depth exceeded 16, possible cycle in: {}", input));
        }

        let Some(calc_start) = input.find("calc(") else {
            return Ok(input.to_string());
        };

        let before = &input[..calc_start];
        let rest = &input[calc_start + 5..]; // skip "calc("

        let (args_str, after) = Self::extract_balanced_parens(rest)?;
        let args = Self::split_args(&args_str)?;

        if args.is_empty() {
            return Err("calc() requires at least a template name".to_string());
        }

        let template_name = args[0].trim();
        let template = self.templates.get(template_name)
            .ok_or_else(|| format!("Unknown calc template: '{}'", template_name))?;

        let template_args = &args[1..];
        if template_args.len() != template.params.len() {
            return Err(format!(
                "calc({}) expects {} arguments, got {}",
                template_name, template.params.len(), template_args.len()
            ));
        }

        let mut result = template.formula.clone();
        for (param, arg) in template.params.iter().zip(template_args.iter()) {
            let placeholder = format!("{{{}}}", param);
            result = result.replace(&placeholder, arg.trim());
        }

        let expanded = format!("{}{}{}", before, result, after);
        self.expand_recursive(&expanded, depth + 1)
    }

    fn extract_balanced_parens(input: &str) -> Result<(String, &str), String> {
        let mut depth = 1;
        let mut end = 0;
        for (i, c) in input.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = i;
                        break;
                    }
                }
                _ => {}
            }
        }
        if depth != 0 {
            return Err("Unmatched parenthesis in calc() expression".to_string());
        }
        Ok((input[..end].to_string(), &input[end + 1..]))
    }

    fn split_args(input: &str) -> Result<Vec<String>, String> {
        let mut args = Vec::new();
        let mut current = String::new();
        let mut depth = 0;

        for c in input.chars() {
            match c {
                '(' => {
                    depth += 1;
                    current.push(c);
                }
                ')' => {
                    depth -= 1;
                    current.push(c);
                }
                ',' if depth == 0 => {
                    args.push(current.clone());
                    current.clear();
                }
                _ => current.push(c),
            }
        }

        if !current.is_empty() || args.is_empty() {
            args.push(current);
        }

        Ok(args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_calc() {
        let reg = CalcRegistry::new();
        assert_eq!(reg.expand("stat(foo) + 1").unwrap(), "stat(foo) + 1");
    }

    #[test]
    fn test_simple_expand() {
        let mut reg = CalcRegistry::new();
        reg.add("double".to_string(), vec!["x".to_string()], "{x} * 2".to_string());
        assert_eq!(reg.expand("calc(double, 5)").unwrap(), "5 * 2");
    }

    #[test]
    fn test_nested_args() {
        let mut reg = CalcRegistry::new();
        reg.add("wrap".to_string(), vec!["x".to_string()], "({x})".to_string());
        assert_eq!(reg.expand("calc(wrap, stat(foo) + 1)").unwrap(), "(stat(foo) + 1)");
    }

    #[test]
    fn test_multiple_params() {
        let mut reg = CalcRegistry::new();
        reg.add("fim".to_string(),
            vec!["flat".to_string(), "inc".to_string(), "more".to_string()],
            "stat({flat}) * (1 + stat({inc})) * stat({more})".to_string(),
        );
        let result = reg.expand("calc(fim, hp_flat, hp_inc, hp_more)").unwrap();
        assert_eq!(result, "stat(hp_flat) * (1 + stat(hp_inc)) * stat(hp_more)");
    }
}
