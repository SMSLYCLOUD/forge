use anyhow::Result;
use std::io::Write;
use std::process::Command;

#[derive(Clone, Debug)]
pub enum Language {
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Go,
    Html,
    Css,
    Json,
    Other(String),
}

pub struct Formatter;

impl Formatter {
    pub fn format(content: &str, lang: Language) -> Result<String> {
        match &lang {
            Language::Rust => Self::format_rust(content),
            Language::JavaScript
            | Language::TypeScript
            | Language::Html
            | Language::Css
            | Language::Json => Self::format_prettier(content, &lang),
            Language::Python => Self::format_black(content),
            Language::Go => Self::format_gofmt(content),
            _ => Ok(content.to_string()),
        }
    }

    fn format_rust(content: &str) -> Result<String> {
        let mut child = Command::new("rustfmt")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(content.as_bytes())?;
        }

        let output = child.wait_with_output()?;
        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?)
        } else {
            // Return original content if formatting failed
            // In a real app we might return an error or log it
            Ok(content.to_string())
        }
    }

    fn format_prettier(content: &str, lang: &Language) -> Result<String> {
        let parser = match lang {
            Language::JavaScript => "babel",
            Language::TypeScript => "typescript",
            Language::Html => "html",
            Language::Css => "css",
            Language::Json => "json",
            _ => "babel",
        };

        let mut child = Command::new("prettier")
            .arg("--parser")
            .arg(parser)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(content.as_bytes())?;
        }

        let output = child.wait_with_output()?;
        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?)
        } else {
            Ok(content.to_string())
        }
    }

    fn format_black(content: &str) -> Result<String> {
        let mut child = Command::new("black")
            .arg("-")
            .arg("--quiet")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(content.as_bytes())?;
        }

        let output = child.wait_with_output()?;
        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?)
        } else {
            Ok(content.to_string())
        }
    }

    fn format_gofmt(content: &str) -> Result<String> {
        let mut child = Command::new("gofmt")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(content.as_bytes())?;
        }

        let output = child.wait_with_output()?;
        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?)
        } else {
            Ok(content.to_string())
        }
    }
}
