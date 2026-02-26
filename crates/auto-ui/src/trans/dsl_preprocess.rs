//! DSL Preprocessor for Auto UI
//!
//! MVP: Transforms DSL-style syntax to standard Auto language syntax.
//!
//! Transformations:
//! - `msg X { }` → `enum X { }`
//! - `widget X { model { } view { } on { } style { } }` → `type X { fields fn view() fn on() }`
//! - `button + { }` → `button "+" { }`
//! - `h2 > ${x}` → `h2 \`${x}\`
//! - `.count += 1` → `self.count += 1`

use regex::Regex;

/// MVP: Transform DSL syntax to standard Auto syntax
pub fn dsl_preprocess(source: &str) -> String {
    let mut result = source.to_string();

    result = remove_dsl_comments(&result);
    result = add_widget_spec(&result);
    result = add_prelude(&result);
    result = transform_msg_to_enum(&result);
    result = transform_widget_to_type(&result);
    result = transform_app_to_main(&result);
    result = transform_inline_buttons(&result);
    result = transform_content_shorthand(&result);
    result = transform_dot_field(&result);

    result
}

fn remove_dsl_comments(source: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let mut result = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("//") && !trimmed.contains("use ") {
            continue;
        }
        result.push(line);
    }

    result.join("\n")
}

fn add_widget_spec(source: &str) -> String {
    // Add spec Widget definition if not present
    if !source.contains("spec Widget") {
        format!("spec Widget {{}}\n\n{}", source)
    } else {
        source.to_string()
    }
}

fn add_prelude(source: &str) -> String {
    if source.contains("use auto.ui:") {
        source.to_string()
    } else {
        format!(
            "use auto.ui: View, app, center, col, button, h2\n\n{}",
            source
        )
    }
}

fn transform_msg_to_enum(source: &str) -> String {
    let re = Regex::new(r"msg\s+(\w+)\s*\{").unwrap();
    re.replace_all(source, "enum $1 {").to_string()
}

fn transform_widget_to_type(source: &str) -> String {
    let widget_re = Regex::new(r"widget\s+(\w+)\s*\{").unwrap();

    let mut result = source.to_string();

    while let Some(caps) = widget_re.captures(&result) {
        let widget_name = caps.get(1).unwrap().as_str();
        let match_str = caps.get(0).unwrap().as_str();
        let start = caps.get(0).unwrap().start();

        let brace_start = result[start..].find('{').unwrap() + start;
        let brace_end = find_matching_brace(&result, brace_start);

        let widget_body = &result[brace_start + 1..brace_end];

        let transformed = transform_widget_body(widget_name, widget_body);

        // Replace the entire widget block with transformed type block
        result = format!(
            "{}type {} {{\n{}}}\n{}",
            &result[..start],
            widget_name,
            transformed,
            &result[brace_end + 1..]
        );
    }

    result
}

fn transform_widget_body(name: &str, body: &str) -> String {
    let model = extract_block(body, "model");
    let view = extract_block(body, "view");
    let on = extract_block(body, "on");
    let style = extract_block(body, "style");

    let mut result = String::new();

    if let Some(m) = model {
        for line in m.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with("//") {
                result.push_str(&format!("    {}\n", trimmed));
            }
        }
        result.push('\n');
    }

    if let Some(s) = style {
        let style_content = extract_style_content(&s);
        result.push_str(&format!("    _style str = \"{}\"\n\n", style_content));
    }

    if let Some(v) = view {
        result.push_str("    fn view() View {\n");
        let indented = reindent_block(&v, 8);
        result.push_str(&indented);
        result.push_str("    }\n\n");
    }

    if let Some(_o) = on {
        result.push_str(&format!("    fn on(ev {}Msg) {{\n", name));
        result.push_str(
            "        // TODO: Pattern matching with enum variants requires auto-lang parser fix\n",
        );
        result.push_str("    }\n");
    }

    result
}

fn reindent_block(block: &str, base_indent: usize) -> String {
    let lines: Vec<&str> = block.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    let min_indent = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);

    let mut result = String::new();
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            result.push('\n');
        } else {
            let current_indent = line.len() - line.trim_start().len();
            let relative_indent = current_indent.saturating_sub(min_indent);
            let new_indent = base_indent + relative_indent;
            result.push_str(&format!("{}{}\n", " ".repeat(new_indent), trimmed));
        }
    }
    result
}

fn extract_block(body: &str, block_name: &str) -> Option<String> {
    let pattern = format!(r"{}\s*\{{", block_name);
    let re = Regex::new(&pattern).unwrap();

    if let Some(caps) = re.captures(body) {
        let start = caps.get(0).unwrap().start();
        let brace_start = body[start..].find('{').unwrap() + start;
        let brace_end = find_matching_brace(body, brace_start);
        return Some(body[brace_start + 1..brace_end].to_string());
    }
    None
}

fn extract_style_content(style_block: &str) -> String {
    let lines: Vec<&str> = style_block
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect();
    lines.join(" ")
}

fn transform_inline_buttons(source: &str) -> String {
    let re = Regex::new(r"button\s+([^\{\s]+)\s*\{").unwrap();
    re.replace_all(source, r#"button "$1" {"#).to_string()
}

fn transform_content_shorthand(source: &str) -> String {
    let re = Regex::new(r"(\w+)\s*>\s*([^\n]+)").unwrap();
    let result = re.replace_all(source, r#"$1 "$2""#).to_string();
    result
}

fn transform_dot_field(source: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_alphabetic() {
            let prev_is_word_char = if i > 0 {
                chars[i - 1].is_ascii_alphanumeric() || chars[i - 1] == '_'
            } else {
                false
            };

            if !prev_is_word_char {
                result.push_str("self.");
                i += 1;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    result.push(chars[i]);
                    i += 1;
                }
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

fn transform_app_to_main(source: &str) -> String {
    let app_re = Regex::new(r"app\s+(\w+)\s*\{").unwrap();

    let mut result = source.to_string();

    while let Some(caps) = app_re.captures(&result) {
        let app_name = caps.get(1).unwrap().as_str();
        let start = caps.get(0).unwrap().start();
        let brace_start = result[start..].find('{').unwrap() + start;
        let brace_end = find_matching_brace(&result, brace_start);
        let app_body = &result[brace_start..=brace_end];

        result = format!(
            "{}fn main() {{\n    app(\"{}\") {}\n}}\n{}",
            &result[..start],
            app_name,
            app_body,
            &result[brace_end + 1..]
        );
    }

    result
}

fn find_matching_brace(source: &str, start: usize) -> usize {
    let chars: Vec<char> = source.chars().collect();
    let mut depth = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, &c) in chars.iter().enumerate().skip(start) {
        if escape_next {
            escape_next = false;
            continue;
        }

        match c {
            '\\' => escape_next = true,
            '"' | '\'' if !in_string => in_string = true,
            '"' | '\'' if in_string => in_string = false,
            '{' if !in_string => depth += 1,
            '}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    return i;
                }
            }
            _ => {}
        }
    }

    source.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_msg_to_enum() {
        let input = "msg CounterMsg { Inc, Dec }";
        let output = transform_msg_to_enum(input);
        assert_eq!(output, "enum CounterMsg { Inc, Dec }");
    }

    #[test]
    fn test_transform_inline_buttons() {
        let input = "button + { onclick: Inc }";
        let output = transform_inline_buttons(input);
        assert_eq!(output, r#"button "+" { onclick: Inc }"#);
    }

    #[test]
    fn test_transform_content_shorthand() {
        let input = "h2 > Current Count: ${count}";
        let output = transform_content_shorthand(input);
        assert_eq!(output, r#"h2 "Current Count: ${count}""#);
    }

    #[test]
    fn test_transform_dot_field() {
        assert_eq!(transform_dot_field(".count += 1"), "self.count += 1");
        assert_eq!(transform_dot_field(".count -= 1"), "self.count -= 1");
        assert_eq!(transform_dot_field(".count = 0"), "self.count = 0");
    }

    #[test]
    fn test_full_preprocess() {
        let input = r#"
msg CounterMsg {
    Inc
    Dec
}

widget Counter {
    model {
        count int
    }

    view {
        col {
            button + { onclick: CounterMsg.Inc }
            h2 > Count: ${count}
            button - { onclick: CounterMsg.Dec }
        }
    }

    on {
        CounterMsg.Inc => { .count += 1 }
        CounterMsg.Dec => { .count -= 1 }
    }
}
"#;

        let output = dsl_preprocess(input);

        assert!(output.contains("enum CounterMsg"));
        assert!(output.contains("type Counter {"));
        assert!(output.contains("fn view() View"));
        assert!(output.contains("fn on(ev CounterMsg)"));
        assert!(output.contains("button \"+\""));
        assert!(output.contains(r#"h2 "Count: ${count}""#));
        assert!(output.contains("TODO: Pattern matching"));
    }
}
