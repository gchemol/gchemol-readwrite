# gchemol-readwrite

[![Crates.io](https://img.shields.io/crates/v/gchemol-readwrite.svg)](https://crates.io/crates/gchemol-readwrite)
[![Documentation](https://docs.rs/gchemol-readwrite/badge.svg)](https://docs.rs/gchemol-readwrite)
[![License](https://img.shields.io/crates/l/gchemol-readwrite.svg)](LICENSE)

A Rust library for reading and writing chemical file formats, built on top of the `gchemol-core` molecular manipulation library.

## Features

- Support for multiple chemical file formats:
  - XYZ (.xyz)
  - CIF (.cif)
  - MOL2 (.mol2)
  - PDB (.pdb)
  - POSCAR/CONTCAR (VASP, *.vasp, POSCAR-xx)
  - Gaussian (.gjf)
  - And more...
- Powerful template engine for custom molecular file format generation using Jinja2-like syntax
- Integration with `gchemol-core` for molecular manipulation and analysis
- Extensible architecture for adding new file formats

## Supported File Formats

| Format | Extensions | Read | Write |
|--------|------------|------|-------|
| XYZ | .xyz | ✅ | ✅ |
| Extended XYZ | .exyz | ✅ | ✅ |
| CIF | .cif | ✅ | ✅ |
| MOL2 | .mol2 | ✅ | ✅ |
| PDB | .pdb | ✅ | ✅ |
| VASP POSCAR | POSCAR, CONTCAR | ✅ | ✅ |
| Gaussian | .log, .out | ✅ | ❌ |
| SDF | .sdf | ✅ | ✅ |
| CML | .cml | ✅ | ✅ |
| Chemical JSON | .cjson | ✅ | ✅ |

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
gchemol-readwrite = "0.1"
```

## Usage

### Reading Molecules

```rust
use gchemol_readwrite::prelude::*;
use gchemol_core::Molecule;

// Read a molecule from a file
let mol = Molecule::from_file("water.xyz")?;

// Or read from a string
let mol = Molecule::from_str("XYZ STRING HERE", "text/xyz")?;
```

### Writing Molecules

```rust
use gchemol_readwrite::prelude::*;
use gchemol_core::Molecule;

// Assuming you have a Molecule object
let mol: Molecule = ...;

// Write to a file
mol.to_file("output.xyz")?;

// Or format as a string
let xyz_string = mol.format_as("text/xyz")?;
```

### Using Templates

The library provides a powerful template engine for generating custom molecular file formats:

```rust
use gchemol_readwrite::prelude::*;
use gchemol_core::Molecule;

let mol = Molecule::from_file("structure.cif")?;

// Render using a Jinja2-like template
let rendered = mol.render_with("template.jinja".as_ref())?;

// Save the rendered output
std::fs::write("output.file", rendered)?;
```

See [Template Usage Documentation](docs/template_usage.md) for more details on the template system.

## Template System

This library includes a flexible template engine based on MiniJinja that allows users to create custom molecular file formats. The template system provides access to molecular data structures and includes a built-in format filter for precise numeric formatting.

Templates can be used to generate input files for various computational chemistry programs like:
- Abacus (STRU)
- VASP (POSCAR)
- And many others

For detailed information about the template system, see [Template Usage Documentation](docs/template_usage.md).

## Documentation

- [API Documentation](https://docs.rs/gchemol-readwrite)
- [Template Usage Guide](docs/template_usage.md)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.