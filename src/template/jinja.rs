// [[file:../../gchemol-readwrite.note::9ecb73b4][9ecb73b4]]
use gut::prelude::*;
use std::path::{Path, PathBuf};

use minijinja::Environment;
// 9ecb73b4 ends here

// [[file:../../gchemol-readwrite.note::b4df5d88][b4df5d88]]
use lazy_regex::*;

static GLOBAL_REX: Lazy<Regex> = lazy_regex!("^ab+$"i);

static FORMAT_SPEC: Lazy<Regex> = lazy_regex!(
    r"(?x)
(?P<align>[<>^])?
(?P<sign>[\+\-])?
(?P<width>\d+)?
((?:\.)(?P<precision>\d+))?
"
);

static FLOAT_NUM: Lazy<Regex> = lazy_regex!(r"^[\-\+]?[0-9]*\.[0-9]+$");
static NUMBER: Lazy<Regex> = lazy_regex!(r"^[\-\+]?[0-9]+$");
// b4df5d88 ends here

// [[file:../../gchemol-readwrite.note::4d5a1513][4d5a1513]]
/// Format number with run time formatting parameters
#[derive(Debug, Clone, Default)]
struct Format {
    align: Option<char>,
    sign: Option<char>,
    width: Option<usize>,
    precision: Option<usize>,
}

impl Format {
    fn parse_from(param: &str) -> Self {
        let mut f = Format::default();

        if let Some(captures) = FORMAT_SPEC.captures(param) {
            f.align = captures.name("align").and_then(|x| x.as_str().chars().next());
            f.sign = captures.name("sign").and_then(|x| x.as_str().chars().next());
            f.width = captures.name("width").and_then(|x| x.as_str().parse().ok());
            f.precision = captures.name("precision").and_then(|x| x.as_str().parse().ok());
        }

        f
    }
}

#[test]
fn test_parse_regex() {
    let captures = FORMAT_SPEC.captures("-12.5").unwrap();
    assert!(captures.name("align").is_none());
    assert!(captures.name("sign").is_some());
    assert!(captures.name("width").is_some());
    assert!(captures.name("precision").is_some());

    let captures = FORMAT_SPEC.captures("^12").unwrap();
    assert!(captures.name("align").is_some());
    assert!(captures.name("width").is_some());

    let captures = FORMAT_SPEC.captures("-12").unwrap();
    assert!(captures.name("align").is_none());
    assert!(captures.name("sign").is_some());
    assert!(captures.name("width").is_some());

    let format = Format::parse_from("-12.5");
    assert_eq!(format.sign, Some('-'));
    assert_eq!(format.width, Some(12));
    assert_eq!(format.precision, Some(5));

    let format = Format::parse_from("-12");
    assert_eq!(format.align, None);
    assert_eq!(format.sign, Some('-'));
    assert_eq!(format.width, Some(12));
    assert_eq!(format.precision, None);

    let format = Format::parse_from("^12");
    assert_eq!(format.sign, None);
    assert_eq!(format.align, Some('^'));
    assert_eq!(format.width, Some(12));
    assert_eq!(format.precision, None);

    assert!(FLOAT_NUM.is_match("12.55"));
    assert!(FLOAT_NUM.is_match("+.55"));
    assert!(FLOAT_NUM.is_match("-1.55"));
    assert!(!FLOAT_NUM.is_match("-1.55a"));
    assert!(NUMBER.is_match("-1"));
}
// 4d5a1513 ends here

// [[file:../../gchemol-readwrite.note::*format][format:1]]
/// format with run time parameters from user
pub fn format_user_value(value: String, code: String) -> String {
    let f = Format::parse_from(&code);

    let width = f.width.unwrap_or(0);
    let precision = f.precision.unwrap_or(width);

    macro_rules! format_value {
        ($v:ident) => {{
            match f.align {
                Some('<') => format!("{value:<width$.precision$}", value = $v, width = width, precision = precision),
                Some('>') => format!("{value:>width$.precision$}", value = $v, width = width, precision = precision),
                Some('^') => format!("{value:^width$.precision$}", value = $v, width = width, precision = precision),
                None => format!("{value:width$.precision$}", value = $v, width = width, precision = precision),
                _ => todo!(),
            }
        }};
    }
    if FLOAT_NUM.is_match(&value) {
        let value: f64 = value.parse().unwrap();
        format_value!(value)
    } else if NUMBER.is_match(&value) {
        let value: usize = value.parse().unwrap();
        format_value!(value)
    } else {
        format_value!(value)
    }
}
// format:1 ends here

// [[file:../../gchemol-readwrite.note::e1b058cf][e1b058cf]]
pub fn render(json_source: &str, template: &str) -> Result<String> {
    let mut env = Environment::new();
    // env.add_filter("format", format_user_value);
    let ctx: serde_json::Value = serde_json::from_str(json_source)?;
    let mut s = env.render_str(template, &ctx)?;
    s.push_str("\n");
    Ok(s)
}

/// Template rendering using minijinja
pub struct Template<'a> {
    env: Environment<'a>,
    src: String,
}

impl<'a> Template<'a> {
    /// Render template as string using vars from `json`.
    pub fn render_json(&self, json: &str) -> Result<String> {
        let ctx: serde_json::Value = serde_json::from_str(json)?;
        let s = self.render(ctx)?;
        Ok(s)
    }

    /// Renders the template into a string using vars from `ctx`.
    pub fn render<S: Serialize>(&self, ctx: S) -> Result<String> {
        let mut s = self
            .env
            .render_str(&self.src, &ctx)
            .map_err(|e| anyhow!("Render molecule failure in tera: {:?}", e))?;
        s.push_str("\n");
        Ok(s)
    }

    /// Load template from file in `path`.
    pub fn try_from_path(path: &Path) -> Result<Self> {
        let src = gut::fs::read_file(path)?;
        let t = Self::from_str(&src);
        Ok(t)
    }

    /// Construct from `src`
    pub fn from_str(src: &str) -> Self {
        let mut env = Environment::new();
        env.add_filter("format", format_user_value);
        Self { env, src: src.to_owned() }
    }
}
// e1b058cf ends here

// [[file:../../gchemol-readwrite.note::7d600fb2][7d600fb2]]
use super::Molecule;

/// render molecule in user defined template
pub(super) fn render_molecule_with(mol: &Molecule, template: &str) -> Result<String> {
    let s = Template::from_str(template).render(super::renderable(mol))?;
    Ok(s)
}
// 7d600fb2 ends here
