// imports

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*imports][imports:1]]
use handlebars::*;

use super::*;
use gchemol_core::Molecule;
// imports:1 ends here

// format helper

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*format helper][format helper:1]]
// https://docs.rs/handlebars/1.0.0/handlebars/trait.HelperDef.html
// define a helper for formatting string or number
fn format(
    h: &Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    // get positional parameter from helper or throw an error
    let param = h
        .param(0)
        .ok_or(RenderError::new("Param 0 is required for format helper."))?;

    // get keyword parameters
    let width = h.hash_get("width").and_then(|v| v.value().as_u64());
    let prec = h.hash_get("prec").and_then(|v| v.value().as_u64());
    let align = h.hash_get("align").and_then(|v| v.value().as_str());

    // format string
    if param.value().is_string() {
        let v = param.value().as_str().ok_or(RenderError::new("param 0: not str"))?;
        let width = width.unwrap_or(0) as usize;
        let rendered = if let Some(align) = align {
            match align {
                "center" => format!("{:^width$}", v, width = width),
                "right" => format!("{:<width$}", v, width = width),
                "left" => format!("{:>width$}", v, width = width),
                _ => format!("{:width$}", v, width = width),
            }
        } else {
            format!("{:width$}", v, width = width)
        };
        out.write(rendered.as_ref())?;

        // format number
    } else if param.value().is_number() || param.value().is_f64() {
        let num: f64 = param
            .value()
            .as_f64()
            .ok_or(RenderError::new("param 0: not f64 number"))?;

        let width = width.unwrap_or(18) as usize;
        let prec = prec.unwrap_or(8) as usize;
        let rendered = if let Some(align) = align {
            match align {
                "center" => format!("{:^width$.prec$}", num, width = width, prec = prec),
                "right" => format!("{:<width$.prec$}", num, width = width, prec = prec),
                "left" => format!("{:>width$.prec$}", num, width = width, prec = prec),
                _ => format!("{:width$.prec$}", num, width = width, prec = prec),
            }
        } else {
            format!("{:-width$.prec$}", num, width = width, prec = prec)
        };
        out.write(rendered.as_ref())?;
    } else {
        return Err(RenderError::new("Possible type for param 0: string or number"));
    }

    Ok(())
}
// format helper:1 ends here

// fgt

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*fgt][fgt:1]]
handlebars_helper!(fgt: |x: f64, y: f64| x > y);
// fgt:1 ends here

// core

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*core][core:1]]
/// render molecule in user defined template
pub(super) fn render_molecule_with(mol: &Molecule, template: &str) -> Result<String> {
    let mut h = Handlebars::new();
    h.register_helper("format", Box::new(format));
    h.register_helper("fgt", Box::new(fgt));

    let data = renderable(mol);
    h.render_template(template, &data)
        .map_err(|e| format_err!("Render molecule failure: {}", e))
}
// core:1 ends here

// test

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*test][test:1]]
#[test]
fn test_template_render() {
    use crate::prelude::*;

    let mol = Molecule::from_file("tests/files/mol2/LTL-crysin-ds.mol2").expect("template mol");

    let template = gut::fs::read_file("tests/files/templates/xyz.hbs").expect("template xyz.hbs");
    let x = render_molecule_with(&mol, &template).unwrap();
}
// test:1 ends here
