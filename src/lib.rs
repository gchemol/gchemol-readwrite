// [[file:../gchemol-readwrite.note::0c98a9cf][0c98a9cf]]
//===============================================================================#
//   DESCRIPTION:  basic read & write support for molecular file
//
//       OPTIONS:  ---
//  REQUIREMENTS:  ---
//         NOTES:  ---
//        AUTHOR:  Wenping Guo <ybyygu@gmail.com>
//       LICENCE:  GPL version 3
//       CREATED:  <2018-04-11 Wed 15:42>
//===============================================================================#
// 0c98a9cf ends here

// [[file:../gchemol-readwrite.note::7fbdd0af][7fbdd0af]]
// ignore compiler warnings due to nom macro uses
#[allow(unused)]
pub mod formats;

mod io;
mod template;
// 7fbdd0af ends here

// [[file:../gchemol-readwrite.note::efea89c0][efea89c0]]
pub mod prelude {
    pub use crate::io::FromFile;
    pub use crate::io::StringIO;
    pub use crate::io::ToFile;
    pub use crate::template::TemplateRendering;
}

pub use crate::formats::describe_backends;
pub use crate::io::{find_files, guess_format_from_path, read, read_all, read_from, write, write_format};
pub use crate::template::to_json;
pub use crate::template::Template;
// efea89c0 ends here
