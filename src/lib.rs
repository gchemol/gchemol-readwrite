// header

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*header][header:1]]
//===============================================================================#
//   DESCRIPTION:  basic read & write support for molecular file
//
//       OPTIONS:  ---
//  REQUIREMENTS:  ---
//         NOTES:  ---
//        AUTHOR:  Wenping Guo <ybyygu@gmail.com>
//       LICENCE:  GPL version 3
//       CREATED:  <2018-04-11 Wed 15:42>
//       UPDATED:  <2020-01-31 Fri 16:28>
//===============================================================================#
// header:1 ends here

// mods

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*mods][mods:1]]
// ignore compiler warnings due to nom macro uses
#[allow(unused)]
mod formats;
mod template;

mod io;
// mods:1 ends here

// exports

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*exports][exports:1]]
pub mod prelude {
    pub use crate::io::FromFile;
    pub use crate::io::StringIO;
    pub use crate::io::ToFile;
    pub use crate::template::TemplateRendering;
}

pub use crate::io::{read, read_all, read_from, write, write_format};
// exports:1 ends here
