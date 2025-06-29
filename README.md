# Scaffer-RS

A Rust clone of [scaffer](https://github.com/vivainio/scaffer) - yet another scaffolding tool that actually does what you want.

Unlike many other scaffolding tools (like cookiecutter, angular schematics etc), this one:

* **Uses working code as templates** - You only need to rename symbols in your live code and it will still compile and/or run. You can modify and test templates naturally without having to remaster the template for publication.
* **No configuration required** - Just place the files somewhere and use.
* **Supports templates from your own repo/source tree**
* **Multiple templates per directory** - Unlike cookiecutter, you can have as many templates as you want in a single tree.
* **Turing complete** - You can add `scaffer_init.py` to your template for advanced logic (coming soon).
* **Not implemented in Node** - Fast, safe, and reliable Rust implementation.

## Installation

```bash
cargo install scaffer-rs
```

Or build from source:

```bash
git clone https://github.com/yourusername/scaffer-rs
cd scaffer-rs
cargo build --release
```

## Usage

```bash
# Generate code from a template
scaffer g

# Generate with specific template
scaffer g my-template

# Generate with pre-defined variables
scaffer g my-template -v project=my-app -v author="John Doe"

# Dry run (see what would be created without creating files)
scaffer g my-template --dry

# Force overwrite existing files
scaffer g my-template -f

# Setup scaffer configuration
scaffer setup

# Add current directory as template root
scaffer add

# Create barrel file (index.ts)
scaffer barrel

# Create .gitignore file
scaffer gitignore
```

## Template Logic

If you want to have template variable 'myvar', represent it by one of these in the templates:

- `ScfMyvar` - PascalCase with Scf prefix
- `scf-myvar` - kebab-case with scf prefix  
- `scf.myvar` - dot.case with scf prefix
- `scf_myvar` - snake_case with scf prefix
- `scfmyvar` - flat lowercase with scf prefix
- `SCFMYVAR` - flat uppercase with SCF prefix
- Or uppercase equivalents: `SCF-MYVAR`, `SCF.MYVAR`, `SCF_MYVAR`

The variable should appear at word boundary at least once to be discovered. For example, `fooscfoeoevaroeuoeuoeu` would not be discovered or replaced, to avoid triggering the logic in random strings that may have "scf" characters.

### Example Template File

```rust
// src/ScfProject/scf_project.rs
pub struct ScfProject {
    name: String,
}

impl ScfProject {
    pub fn new() -> Self {
        Self {
            name: "scf-project".to_string(),
        }
    }
}

pub const SCF_PROJECT_VERSION: &str = "1.0.0";
```

When scaffer processes this template with variable `project=my-app`, it becomes:

```rust
// src/MyApp/my_app.rs
pub struct MyApp {
    name: String,
}

impl MyApp {
    pub fn new() -> Self {
        Self {
            name: "my-app".to_string(),
        }
    }
}

pub const MY_APP_VERSION: &str = "1.0.0";
```

Template variables can also be in file and directory names, and behave as you would expect.

## Template Discovery

1. Place your template files somewhere
2. In project root, or any parent directory, put `scaffer.json` that points to directories containing your templates:

```json
{
    "scaffer": ["my/templates", "some/other/templates"]
}
```

You can also add `scaffer_template_urls` to configure templates to be downloaded via HTTP:

```json
{
    "scaffer": ["my/templates", "some/other/templates"],
    "scaffer_template_urls": {
        "rust-api": "https://example.com/rust-api-template.zip",
        "react-component": "https://example.com/react-component.zip"
    }
}
```

You can also put the "scaffer" key in your `package.json` if you don't want to pollute your tree with new files.

## Configuration

Scaffer looks for configuration in the following order:

1. `scaffer.json` in current directory or any parent directory
2. `scaffer` key in `package.json` in current directory or any parent directory  
3. Global configuration in `~/.scaffer.json`

## Examples

See example templates at: https://github.com/vivainio/scaffer-templates

## Commands

### `scaffer g [template]`

Generate code from a template.

**Options:**
- `-v, --var <variable=value>` - Give value to variable
- `-f, --force` - Overwrite files if needed
- `--dry` - Dry run, do not create files

### `scaffer add`

Add current directory as template root in user global scaffer.json.

### `scaffer barrel`

Create `index.ts` barrel file for current directory, exporting all TypeScript modules.

### `scaffer gitignore`

Create a comprehensive `.gitignore` file.

### `scaffer setup`

Interactive setup for scaffer configuration.

## Development

```bash
# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run -- g

# Format code
cargo fmt

# Lint code
cargo clippy
```

## License

MIT License - see LICENSE file for details.

## Differences from Original Scaffer

This Rust implementation provides:

- **Better Performance** - Rust's speed and memory safety
- **Cross-platform** - Single binary that works everywhere
- **Type Safety** - Compile-time guarantees
- **Modern CLI** - Built with clap for better UX
- **Async Support** - Non-blocking template downloads
- **Comprehensive Testing** - Unit tests for all functionality

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## Roadmap

- [ ] Python script execution for `scaffer_init.py`
- [ ] Template validation
- [ ] Template caching
- [ ] Plugin system
- [ ] IDE integrations
- [ ] Template marketplace 