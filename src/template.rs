// imports

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*imports][imports:1]]
use guts::prelude::*;

use gchemol_core::Molecule;
// imports:1 ends here

// mods

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*mods][mods:1]]
mod hbs;
// mods:1 ends here

// traits

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*traits][traits:1]]
/// Render molecule in user defined format
pub trait TemplateRendering {
    /// Render molecule with user defined template
    fn render_with(&self, template: &str) -> Result<String>;
}
// traits:1 ends here
