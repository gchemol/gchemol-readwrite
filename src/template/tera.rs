// imports

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*imports][imports:1]]
use super::*;
use gchemol_core::Molecule;

use ::tera::Value;
// imports:1 ends here

// format

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*format][format:1]]
fn format_filter(value: &Value, args: &std::collections::HashMap<String, Value>) -> ::tera::Result<Value> {
    // get keyword parameters
    let width = args.get("width").and_then(|v| v.as_u64());
    let prec = args.get("prec").and_then(|v| v.as_u64());
    let align = args.get("align").and_then(|v| v.as_str()).unwrap_or("");
    match value {
        Value::Number(n) => {
            let w = width.unwrap_or(18) as usize;
            let p = prec.unwrap_or(8) as usize;
            let s = match align {
                "left" => format!("{:<-width$.prec$}", n, width = w, prec = p),
                "right" => format!("{:>-width$.prec$}", n, width = w, prec = p),
                "center" => format!("{:^-width$.prec$}", n, width = w, prec = p),
                _ => format!("{:-width$.prec$}", n, width = w, prec = p),
            };
            Ok(s.into())
        }
        Value::String(n) => {
            let w = width.unwrap_or(0) as usize;
            let s = match align {
                "left" => format!("{:<-width$}", n, width = w),
                "right" => format!("{:>-width$}", n, width = w),
                "center" => format!("{:^-width$}", n, width = w),
                _ => format!("{:-width$}", n, width = w),
            };
            Ok(s.into())
        }
        _ => {
            let s = format!("{:}", value);
            Ok(s.into())
        }
    }
}
// format:1 ends here

// impl

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*impl][impl:1]]
/// render molecule in user defined template
pub(super) fn render_molecule_with(mol: &Molecule, template: &str) -> Result<String> {
    use ::tera::{Context, Tera};

    let mut tera = Tera::default();
    tera.add_raw_template("molecule", template)?;
    tera.register_filter("format", format_filter);

    let data = renderable(mol);
    let context = Context::from_value(data)?;
    tera.render("molecule", &context)
        .map_err(|e| format_err!("Render molecule failure in tera: {:?}", e))
}
// impl:1 ends here
