# Rockide for Zed

Minecraft Bedrock Edition addon development support for [Zed](https://zed.dev).

Powered by the [Rockide Language Server](https://github.com/rockide/language-server).

## Features

- **Completions** — context-aware suggestions for entities, blocks, items, and more
- **Go to Definition** — jump to referenced files and identifiers
- **Rename** — rename symbols across files
- **Hover** — documentation and type info on hover
- **Signature Help** — parameter hints for Molang functions
- **Semantic Tokens** — syntax highlighting for Molang and `.lang` color codes
- **JSON Schema Validation** — diagnostics for Bedrock addon JSON files (57 schemas)
- **`.lang` File Support** — syntax highlighting for Minecraft localization files

## Supported File Types

| Language | Extensions |
|---|---|
| JSON / JSONC | `.json` (behavior packs, resource packs) |
| Minecraft Lang | `.lang` (localization files) |

## Installation

Install **Rockide** from the Zed extension marketplace.

The language server binary is downloaded automatically from GitHub releases on first use.

## Configuration

### Project Paths

Configure behavior/resource pack paths in your project's `.zed/settings.json`:

```json
{
  "lsp": {
    "rockide": {
      "initialization_options": {
        "projectPaths": {
          "behaviorPack": "./BP",
          "resourcePack": "./RP"
        }
      }
    }
  }
}
```

### Semantic Token Colors

To enable Minecraft color code highlighting, add to your Zed settings:

```json
{
  "semantic_tokens": "combined",
  "semantic_token_rules": {
    "colorBlack": { "foreground_color": "#000000" },
    "colorDarkBlue": { "foreground_color": "#0000AA" },
    "colorDarkGreen": { "foreground_color": "#00AA00" },
    "colorDarkAqua": { "foreground_color": "#00AAAA" },
    "colorDarkRed": { "foreground_color": "#AA0000" },
    "colorDarkPurple": { "foreground_color": "#AA00AA" },
    "colorGold": { "foreground_color": "#FFAA00" },
    "colorGray": { "foreground_color": "#AAAAAA" },
    "colorDarkGray": { "foreground_color": "#555555" },
    "colorBlue": { "foreground_color": "#5555FF" },
    "colorGreen": { "foreground_color": "#55FF55" },
    "colorAqua": { "foreground_color": "#55FFFF" },
    "colorRed": { "foreground_color": "#FF5555" },
    "colorLightPurple": { "foreground_color": "#FF55FF" },
    "colorYellow": { "foreground_color": "#FFFF55" },
    "colorWhite": { "foreground_color": "#FFFFFF" },
    "colorMinecoinGold": { "foreground_color": "#DDD605" },
    "colorMaterialQuartz": { "foreground_color": "#E3D4D1" },
    "colorMaterialIron": { "foreground_color": "#CECACA" },
    "colorMaterialNetherite": { "foreground_color": "#443A3B" },
    "colorMaterialRedstone": { "foreground_color": "#971607" },
    "colorMaterialCopper": { "foreground_color": "#B4684D" },
    "colorMaterialGold": { "foreground_color": "#DEB12D" },
    "colorMaterialEmerald": { "foreground_color": "#47A036" },
    "colorMaterialDiamond": { "foreground_color": "#2CBAA8" },
    "colorMaterialLapis": { "foreground_color": "#21497B" },
    "colorMaterialAmethyst": { "foreground_color": "#9A5CC6" },
    "colorMaterialResin": { "foreground_color": "#FC7703" }
  }
}
```

## License

MIT
