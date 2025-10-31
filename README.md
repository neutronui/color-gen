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

<details>
<summary>FAQ</summary>

> Yes I know the code is terrible...

</details>