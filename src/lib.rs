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
//       UPDATED:  <2020-01-21 Tue 17:23>
//===============================================================================#
// header:1 ends here

// mods

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*mods][mods:1]]
// ignore compiler warnings due to nom macro uses
#[allow(unused)]
mod formats;
mod template;

mod io;

pub mod prelude {
    pub use crate::io::FromFile;
    pub use crate::io::ToFile;
}

pub use crate::io::{read, read_all, write};

#[macro_use]
extern crate handlebars;

// FIXME: why must be located in lib.rs?
handlebars_helper!(fgt: |x: f64, y: f64| x > y);
// mods:1 ends here
