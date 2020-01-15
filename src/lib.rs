// lib.rs
// :PROPERTIES:
// :header-args: :tangle src/lib.rs
// :END:

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*lib.rs][lib.rs:1]]
// ignore compiler warnings due to nom macro uses
#[allow(unused)]
mod formats;

mod io;

pub mod prelude {
    pub use crate::io::FromFile;
    pub use crate::io::ToFile;
}

pub use crate::io::{read, read_all, write};
// lib.rs:1 ends here
