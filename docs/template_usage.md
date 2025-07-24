# Template Usage Documentation

This document provides instructions on how to use the templating system for generating custom molecular file formats.

## Future Direction

The templating system is moving towards standardization on Jinja2 as the primary template engine. Handlebars (.hbs) and Tera (.tera) support will be maintained only for backward compatibility with existing templates. For new template development, it is recommended to use the Jinja2 syntax with a `.jinja` file extension.

For detailed information about the Jinja2 syntax supported by our system, please refer to the [MiniJinja documentation](https://docs.rs/minijinja/latest/minijinja/).

## Overview

The templating system allows you to convert molecular data into various custom formats using template files. The system supports multiple template engines, including Jinja2 (default), Handlebars, and Tera.

## Template Selection

The template engine is automatically selected based on the file extension of the template:
- `.hbs` files use Handlebars
- `.tera` files use Tera
- All other files use Jinja2 (including `.jinja` files)

## Basic Usage

To use a template with a molecule:

```rust
use gchemol_core::Molecule;
use gchemol_readwrite::prelude::*;

// Load a molecule from a file
let mol = Molecule::from_file("path/to/your/molecule.xyz")?;

// Render the molecule using a template
let template_path = "path/to/your/template.jinja";
let rendered_content = mol.render_with(template_path.as_ref())?;

// Save the rendered content to a file
std::fs::write("output.file", &rendered_content)?;
```

## Available Data in Templates

When creating templates, you have access to the following molecular data:

### Molecule Data
- `molecule.title`: The title of the molecule
- `molecule.number_of_atoms`: Total number of atoms
- `molecule.number_of_bonds`: Total number of bonds
- `molecule.number_of_species`: Number of different element types

### Unit Cell Data
- `molecule.unit_cell`: Lattice information (if periodic)
  - `unit_cell.a`, `unit_cell.b`, `unit_cell.c`: Lattice lengths
  - `unit_cell.alpha`, `unit_cell.beta`, `unit_cell.gamma`: Lattice angles
  - `unit_cell.va`, `unit_cell.vb`, `unit_cell.vc`: Lattice vectors

### Species Data
- `molecule.species`: List of element types
  - `species.element_symbol`: Element symbol (e.g., "Si", "O")
  - `species.element_number`: Atomic number
  - `species.number_of_atoms`: Number of atoms of this element

### Atom Data
- `molecule.atoms`: List of atoms
  - `atom.index`: Atom index (1-based)
  - `atom.symbol`: Element symbol
  - `atom.number`: Atomic number
  - `atom.x`, `atom.y`, `atom.z`: Cartesian coordinates
  - `atom.fx`, `atom.fy`, `atom.fz`: Fractional coordinates (if periodic)
  - `atom.freezing`: Array of three boolean values indicating if the atom is frozen in x, y, z directions

## Built-in Filters

The templating system provides a custom `format` filter for number formatting:

```jinja
{{ atom.x | format("12.6") }}  // Format with width 12 and 6 decimal places
{{ atom.number | format("6") }}  // Format with width 6
```

The format specification follows the pattern: `[align][sign][width][.precision]`
- `align`: `<` (left), `>` (right), `^` (center)
- `sign`: `+` (always show sign), `-` (show sign for negative only)
- `width`: Minimum field width
- `precision`: Number of digits after decimal point

## Template Examples

### Simple XYZ-like Template
```jinja
{{ molecule.number_of_atoms }}
{{ molecule.title }}
{% for atom in molecule.atoms %}
{{ atom.symbol | format("3") }} {{ atom.x | format("12.6") }} {{ atom.y | format("12.6") }} {{ atom.z | format("12.6") }}
{% endfor %}
```

### Template with Unit Cell Information
```jinja
{% if molecule.unit_cell %}
Lattice vectors:
{{ molecule.unit_cell.va[0] | format("10.4") }} {{ molecule.unit_cell.va[1] | format("10.4") }} {{ molecule.unit_cell.va[2] | format("10.4") }}
{{ molecule.unit_cell.vb[0] | format("10.4") }} {{ molecule.unit_cell.vb[1] | format("10.4") }} {{ molecule.unit_cell.vb[2] | format("10.4") }}
{{ molecule.unit_cell.vc[0] | format("10.4") }} {{ molecule.unit_cell.vc[1] | format("10.4") }} {{ molecule.unit_cell.vc[2] | format("10.4") }}
{% endif %}
```

### Template with Freezing Information
```jinja
{% for atom in molecule.atoms %}
{{ atom.symbol | format("3") }} {{ atom.x | format("12.6") }} {{ atom.y | format("12.6") }} {{ atom.z | format("12.6") }} {{ (not atom.freezing[0]) | int }} {{ (not atom.freezing[1]) | int }} {{ (not atom.freezing[2]) | int }}
{% endfor %}
```

## Creating Custom Templates

When creating custom templates, consider the following:

1. **Data Structure**: Familiarize yourself with the available data structure described above
2. **Format Requirements**: Understand the specific requirements of the output format you're targeting
3. **Conditional Logic**: Use conditional statements to handle optional sections
4. **Loops**: Use loops to iterate over atoms, species, or other lists
5. **Formatting**: Use the `format` filter to ensure proper alignment and precision

## Testing Templates

You can test your templates by creating a simple test in `tests/template.rs`:

```rust
#[test]
fn test_your_template() -> Result<()> {
    let mol = Molecule::from_file("path/to/test/molecule.xyz")?;
    let tpl = "path/to/your/template.jinja";
    let s = mol.render_with(tpl.as_ref())?;
    
    // Save to file for inspection
    std::fs::write("test_output.file", &s)?;
    
    // Add assertions to verify correctness
    assert!(s.contains("expected content"));
    
    Ok(())
}
```

## Best Practices

1. **Consistent Formatting**: Use the `format` filter to ensure consistent output formatting
2. **Conditional Sections**: Use conditional logic to include optional sections only when relevant data is present
3. **Clear Comments**: Add comments to your templates to explain complex sections
4. **Testing**: Always test your templates with various molecular data to ensure they work correctly
5. **Documentation**: Document any custom parameters or expected data structures for your templates