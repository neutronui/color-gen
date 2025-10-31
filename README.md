# Substrate Color Palette Generator

## Getting Started

### Install
```sh
cargo install --git https://github.com/neutronui/color-gen.git
```

### Usage
```sh
substrate --config path/to/config.json
```

### Resolve Tokens from JSON (optional)
```sh
substrate --tokens path/to/tokens.json [--tokens-out-dir out] [--tokens-selector :root] [--tokens-prefix app]
```

### JavaScript transforms (experimental)

You can add custom transform functions in JavaScript and call them by name from the transform pipeline (requires building with the `js` feature).

- Build with JS feature:
  - Cargo build: `cargo build --features js`
  - Cargo run: `cargo run --features js -- ...`
- Load plugin files at runtime with `--tokens-plugin <file.js>`. All top-level functions in those files are available as transform names.
- Example plugin files are in `examples/plugins/` (e.g., `math.js`, `color.js`).

Notes and limitations for now:
- The current JSON input format treats objects as plain objects; JSON-encoded transform expressions are not yet supported. Transforms (including JS ones) are executed by the Rust pipeline and are best constructed programmatically at the moment.
- JS functions receive `(input, args, ctx)` and should return a JSON-serializable result. A Dimension can be returned as `{ type: 'dimension', value, unit }`.
- Security: plugin code runs locally in an embedded JS engine (Boa). Review plugins before use.

### Configuration
```json
{
  "outDir": "path/to/output/directory",
  "palettes": [
    {
      "name": "example-palette",
      "default": true,
      "prefix": "ex",
      "tones": [50,100,200,300,400,500,600,700,800,900,950],
      "hues": {
        "blue": "#0000ff",
        "red": "#ff0000",
        "green": "#00ff00"
      },
      "variants": {
        "brand": "blue",
        "danger": "red",
        "success": "green"
      }
    },
  ]
}
```

## Token JSON schema (for `--tokens`)

The `--tokens` mode accepts a simple JSON map from token path to a Token object. This is designed for quick theming and CSS output.

Schema:
- Root: object whose keys are token paths like `"spacing.base"` or `"color.brand.50"`.
- Each value is a Token with fields:
  - `name` (string): must match the key (recommended)
  - `value` (string | number | boolean | object): literal CSS-friendly value
  - `comment` (string, optional)

Example (minimal):
```json
{
  "color.brand.50": { "name": "color.brand.50", "value": "#3500ff" },
  "spacing.base":   { "name": "spacing.base",   "value": "4px" }
}
```

Example (with calc/var and comments):
```json
{
  "color.brand.50": {
    "name": "color.brand.50",
    "value": "#3500ff",
    "comment": "Brand primary color"
  },
  "color.brand.50.text": {
    "name": "color.brand.50.text",
    "value": "#ffffff",
    "comment": "On-color for brand 50"
  },
  "spacing.base": {
    "name": "spacing.base",
    "value": "4px",
    "comment": "Base spacing"
  },
  "spacing.large": {
    "name": "spacing.large",
    "value": "calc(var(--spacing-base) * 4)",
    "comment": "Large spacing derived via calc"
  },
  "font.size.base": {
    "name": "font.size.base",
    "value": "16px"
  },
  "font.size.lg": {
    "name": "font.size.lg",
    "value": "calc(var(--font-size-base) * 1.125)",
    "comment": "Slightly larger font using calc"
  }
}
```

Running with PowerShell on Windows:
```powershell
# Resolve and write to .\\out by default
substrate --tokens .\tmp\tokens.sample.json

# Custom out dir, selector, and prefix
substrate --tokens .\tmp\tokens.sample.json --tokens-out-dir .\tmp\out --tokens-selector ':root' --tokens-prefix 'app'

# With JS plugins (build with feature first)
cargo run --features js -- --tokens .\tokens.sample.json --tokens-out-dir .\tmp\out --tokens-plugin .\examples\plugins\math.js --tokens-plugin .\examples\plugins\color.js
```

Outputs:
- `tokens.css`: CSS Custom Properties under the provided selector. Keys become `--path-with-dashes`.
- `tokens.resolved.json`: Map of token path to resolved string value.

Notes and current limitations:
- In JSON mode, values are treated as literal CSS-ready strings/numbers/booleans/objects. You can author `var(...)` and `calc(...)` directly.
- Advanced typed values (like Dimension) and transform pipelines are currently best constructed via Rust. Future iterations may add a friendlier JSON layer for alias/reference/typed transforms.

<details>
<summary>FAQ</summary>

> Yes I know the code is terrible...

</details>