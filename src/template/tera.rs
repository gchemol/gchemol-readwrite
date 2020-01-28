// imports

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*imports][imports:1]]
use super::*;
use gchemol_core::{Atom, Molecule};
use guts::prelude::*;
// imports:1 ends here

// impl

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*impl][impl:1]]
/// render molecule in user defined template
pub(super) fn render_molecule_with(mol: &Molecule, template: &str) -> Result<String> {
    use ::tera::{Context, Tera};

    let data = renderable(mol);
    let context = Context::from_value(data)?;
    Tera::one_off(template, &context, true).map_err(|e| format_err!("Render molecule failure in tera: {:?}", e))
}
// impl:1 ends here
