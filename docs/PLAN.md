# Rockide for Zed — Porting Plan

## Overview

Port the [rockide-vscode](https://github.com/rockide/rockide-vscode) extension to Zed Editor.
Rockide provides rich editing support for **Minecraft Bedrock Edition** addon development
(JSON behavior/resource packs, Molang expressions, `.lang` localization files).

Under the hood it uses the [rockide language server](https://github.com/rockide/language-server) —
a **Go binary** that communicates over **stdio** using LSP JSON-RPC. This is a perfect fit for
Zed's extension model, which natively supports downloading and launching binary language servers.

---

## Feasibility Assessment

### Will port directly

| Feature | Why |
|---|---|
| Language server integration | Zed extensions download binaries from GitHub releases and launch via stdio — exactly how rockide works |
| Completions | LSP `textDocument/completion` — supported |
| Go to Definition | LSP `textDocument/definition` — supported |
| Rename | LSP `textDocument/rename` + `prepareRename` — supported |
| Hover | LSP `textDocument/hover` — supported |
| Signature Help | LSP `textDocument/signatureHelp` — supported |
| Semantic Tokens | LSP `textDocument/semanticTokens/full` — Zed supports `"semantic_tokens": "combined"` or `"full"` |
| File watching | LSP `workspace/didChangeWatchedFiles` — supported |
| Cross-platform binary | rockide publishes `darwin/linux/windows` x `amd64/arm64` — Zed API provides `current_platform()` |

### Needs adaptation

| Feature | VS Code approach | Zed approach |
|---|---|---|
| Molang syntax highlighting | TextMate grammar (`.tmLanguage.json`) | **Need tree-sitter grammar** — none exists publicly, must create one |
| `.lang` file syntax highlighting | Language configuration only (no grammar) | **Need tree-sitter grammar** — simple key=value format, straightforward to write |
| JSON/JSONC support | Built-in + extension overrides | Zed has built-in JSON + tree-sitter-json; register `.material` as JSON |
| Minecraft color semantic tokens | 28 custom `semanticTokenTypes` with hex colors | Zed `semantic_token_rules` in settings — we provide recommended settings |
| `projectPaths` init option | `initializationOptions` via client config | `language_server_initialization_options` method on Extension trait |

### Needs adaptation (schema validation)

| Feature | VS Code approach | Zed approach |
|---|---|---|
| JSON Schema validation (46 schemas) | `contributes.jsonValidation` maps globs → schema files; VS Code's built-in JSON LS does the validation | Zed's built-in `json-language-server` supports `json.schemas` settings — same engine, different config path. See [Schema Validation Strategy](#schema-validation-strategy) below. |

### Won't port (VS Code-specific, not needed)

| Feature | Reason |
|---|---|
| Auto-update check command | Zed handles extension/LSP lifecycle automatically |
| Custom binary download with SHA-256 verification | Zed's `download_file` API handles downloads; no custom verification needed |

---

## Schema Validation Strategy

**No language server changes required.** Zed uses the same `vscode-json-languageserver` under
the hood and supports configuring custom schemas via `json.schemas` in Zed settings.

### How it works

Zed's JSON language server accepts schema configuration in `.zed/settings.json`:

```json
{
  "lsp": {
    "json-language-server": {
      "settings": {
        "json": {
          "schemas": [
            {
              "fileMatch": ["**/entities/**/*.json"],
              "url": "https://raw.githubusercontent.com/bridge-core/editor-packages/main/packages/minecraftBedrock/schema/entity/main.json"
            }
          ]
        }
      }
    }
  }
}
```

### Implementation approach

1. **Map all 46 schema entries** from rockide-vscode's `package.json` `contributes.jsonValidation`
   section into the Zed `json.schemas` format (glob pattern → raw GitHub URL from bridge-core)
2. **Provide a ready-to-use settings snippet** — generate a complete `.zed/settings.json` with all
   46 schema mappings that users can drop into their project
3. **Document in the extension README** — explain how to enable schema validation
4. **Optionally: publish schemas to [SchemaStore.org](https://www.schemastore.org/)** — if accepted,
   Zed's JSON LS would pick them up automatically without any user configuration

### Schema sources

The schemas come from [bridge-core/editor-packages](https://github.com/bridge-core/editor-packages)
and are already publicly available on GitHub. The rockide-vscode extension fetches them at build
time using `degit`. For Zed, we reference them directly via raw GitHub URLs or bundle them.

### What users get

- **Rockide LSP**: completions, go-to-definition, rename, hover, signature help, semantic tokens
- **Zed's JSON LSP**: schema validation (diagnostics for invalid structure, missing fields, wrong types)
- Both run side by side on the same JSON files — no conflicts

---

## Project Structure

```
rockide-zed-vibers/
├── extension.toml              # Extension manifest
├── Cargo.toml                  # Rust crate config (cdylib → Wasm)
├── src/
│   └── lib.rs                  # Extension trait impl + binary download logic
├── languages/
│   ├── json/
│   │   └── config.toml         # Override: add .material extension to JSON
│   ├── molang/
│   │   ├── config.toml         # Molang language definition
│   │   ├── highlights.scm      # Syntax highlighting queries
│   │   ├── brackets.scm        # Bracket matching
│   │   └── indents.scm         # Auto-indentation
│   └── lang/
│       ├── config.toml         # Minecraft .lang language definition
│       ├── highlights.scm      # Syntax highlighting queries
│       └── brackets.scm        # Bracket matching
├── grammars/
│   ├── tree-sitter-molang/     # Custom tree-sitter grammar (new)
│   │   ├── grammar.js
│   │   └── src/                # Generated parser
│   └── tree-sitter-mclang/     # Custom tree-sitter grammar (new)
│       ├── grammar.js
│       └── src/                # Generated parser
└── docs/
    └── PLAN.md                 # This file
```

---

## Implementation Steps

### Phase 1: Scaffold + Language Server Integration

**Goal**: Get the language server downloading, launching, and communicating with Zed.

1. **Create `extension.toml`**
   ```toml
   id = "rockide"
   name = "Rockide"
   description = "Minecraft Bedrock Edition addon development support."
   version = "0.1.0"
   schema_version = 1
   authors = ["Beltsazar"]
   repository = "https://github.com/rockide/rockide-zed"

   [language_servers.rockide]
   name = "Rockide"
   languages = ["JSON", "JSONC", "Molang", "Minecraft Lang"]

   [language_servers.rockide.language_ids]
   "JSON" = "json"
   "JSONC" = "jsonc"
   "Molang" = "rockide-molang"
   "Minecraft Lang" = "rockide-lang"

   [grammars.molang]
   repository = "https://github.com/rockide/tree-sitter-molang"
   rev = "<commit-sha>"

   [grammars.mclang]
   repository = "https://github.com/rockide/tree-sitter-mclang"
   rev = "<commit-sha>"
   ```

2. **Create `Cargo.toml`**
   ```toml
   [package]
   name = "rockide-zed"
   version = "0.1.0"
   edition = "2021"
   publish = false
   license = "MIT"

   [lib]
   path = "src/lib.rs"
   crate-type = ["cdylib"]

   [dependencies]
   zed_extension_api = "0.7.0"
   ```

3. **Implement `src/lib.rs`** — Follow the Gleam/OmniSharp pattern:
   - Struct `RockideExtension` with `cached_binary_path: Option<String>`
   - `language_server_binary_path()`: check PATH for `rockide` → check cache → download from `rockide/language-server` GitHub releases
   - Platform mapping: `windows/linux/darwin` × `amd64/arm64` → asset name `rockide_{version}_{os}_{arch}.tar.gz`
   - `language_server_command()`: return binary path, no args needed
   - `language_server_initialization_options()`: pass `projectPaths` from Zed settings

### Phase 2: Tree-Sitter Grammars

**Goal**: Create minimal tree-sitter grammars for Molang and `.lang` files.

4. **Create `tree-sitter-molang` grammar**

   Molang is a simple expression language. Core syntax:
   - Prefixed queries: `query.is_sneaking`, `v.my_var`, `math.sin()`
   - Operators: `+`, `-`, `*`, `/`, `==`, `!=`, `<`, `>`, `&&`, `||`, `!`
   - Ternary: `condition ? a : b`
   - Assignment: `v.x = 1.0`
   - Literals: numbers (`1.0`), strings (`'hello'`)
   - Parenthesized expressions, function calls
   - Semicolons as statement separators
   - `return`, `loop`, `for_each`, `break`, `continue`

   This grammar will be hosted in a separate repo (`rockide/tree-sitter-molang`) and referenced by commit SHA in `extension.toml`.

   > **Note**: The LSP provides semantic tokens for Molang, so even a basic grammar is fine — semantic tokens will layer richer highlighting on top.

5. **Create `tree-sitter-mclang` grammar**

   `.lang` files are extremely simple:
   ```
   ## Comment
   key.name=Value text with §acolor §lcodes
   ```
   Grammar needs: comments (`##`), keys (before `=`), values (after `=`), format codes (`§` + char).

6. **Write `highlights.scm` query files** for both grammars.

### Phase 3: Language Configuration

**Goal**: Register file types and configure editor behavior.

7. **`languages/molang/config.toml`**
   ```toml
   name = "Molang"
   grammar = "molang"
   path_suffixes = ["molang"]
   line_comments = ["// "]
   tab_size = 4
   ```

8. **`languages/lang/config.toml`**
   ```toml
   name = "Minecraft Lang"
   grammar = "mclang"
   path_suffixes = ["lang"]
   line_comments = ["## "]
   tab_size = 4
   ```

9. **JSON/JSONC file association** — Register `.material` files as JSON.
   The LSP serves JSON/JSONC files which Zed already supports natively.
   We declare `languages = ["JSON", "JSONC"]` on the language server so
   the rockide LSP activates for those built-in types.

### Phase 4: Semantic Token Styling

**Goal**: Make Minecraft color codes render with correct colors.

10. **Document recommended Zed settings** for semantic token colors:
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
      },
      "semantic_tokens": "combined",
      "semantic_token_rules": {
        "colorRed": { "foreground_color": "#FF5454" },
        "colorGreen": { "foreground_color": "#54FF54" },
        "colorAqua": { "foreground_color": "#54FFFF" }
      }
    }
    ```
    Provide a full settings snippet in the extension README covering all 28 Minecraft color tokens.

### Phase 5: Schema Validation

**Goal**: Port the 46 JSON schema mappings so Zed's built-in JSON LS validates Minecraft Bedrock files.

11. **Extract all `jsonValidation` entries** from rockide-vscode's `package.json` — map each
    `{ fileMatch, url }` pair to the Zed `json.schemas` format
12. **Resolve schema URLs** — convert local schema paths (e.g., `./schemas/entity/main.json`)
    to stable raw GitHub URLs from bridge-core/editor-packages
13. **Generate a `.zed/settings.json` template** — a ready-to-use file with all 46 schema mappings
    that users drop into their Minecraft project
14. **Document in README** — explain how to enable schema validation with one copy-paste

### Phase 6: Testing & Publishing

**Goal**: End-to-end verification and publish to Zed extension marketplace.

15. **Local testing** — Use Zed's "Install Dev Extension" pointed at the project directory
16. **Verify all features** work:
    - Completions in JSON entity/block/item files
    - Go-to-definition for identifiers
    - Rename symbols
    - Hover info on Molang expressions
    - Signature help for Molang functions
    - Semantic token highlighting in `.lang` files
    - Schema validation diagnostics in JSON files
17. **Publish** — Fork `zed-industries/extensions`, add as submodule, submit PR

---

## Dependency Chain

```
Phase 1 (Zed extension scaffold + LSP integration)
    ↓
Phase 2 (tree-sitter grammars)  ← parallel with Phase 1
    ↓
Phase 3 (language config)       ← depends on Phase 2
    ↓
Phase 4 (semantic tokens)       ← depends on Phase 1
    ↓
Phase 5 (schema validation)     ← parallel with Phase 1-4 (just config mapping)
    ↓
Phase 6 (testing & publishing)  ← depends on all above
```

**Phases 1, 2, and 5 can all be developed in parallel.**

---

## Open Questions

1. **Tree-sitter grammar hosting** — Should the Molang and `.lang` grammars live in separate repos under the `rockide` org (required for Zed's `[grammars]` section), or can they be embedded? → Separate repos are required by Zed's build system.

2. **`projectPaths` in Zed** — The language server uses `initializationOptions.projectPaths` to locate BP/RP folders. In Zed, this would come from `language_server_initialization_options()`. Users would configure this in their project's `.zed/settings.json`. Need to verify the Extension trait supports this method.

3. **JSON language server activation** — Zed already has a built-in JSON language server. We need to ensure the rockide LSP can run alongside it without conflicts. The `language_ids` mapping in `extension.toml` should handle this.

4. **`.material` file registration** — Need to confirm if Zed allows extensions to add extra file suffixes to built-in languages (JSON), or if we need to register it as a separate language.

---

## Schema Settings Template

Copy this into your project's `.zed/settings.json` to enable JSON schema validation for all Minecraft Bedrock file types:

```json
{
  "lsp": {
    "json-language-server": {
      "settings": {
        "json": {
          "schemas": [
            {
              "fileMatch": [
                "behavior_pack/aim_assist/categories/**/*.json",
                "*BP/aim_assist/categories/**/*.json",
                "BP_*/aim_assist/categories/**/*.json",
                "*bp/aim_assist/categories/**/*.json",
                "bp_*/aim_assist/categories/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/aimAssistCategories/main.json"
            },
            {
              "fileMatch": [
                "behavior_pack/aim_assist/presets/**/*.json",
                "*BP/aim_assist/presets/**/*.json",
                "BP_*/aim_assist/presets/**/*.json",
                "*bp/aim_assist/presets/**/*.json",
                "bp_*/aim_assist/presets/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/aimAssistPreset/main.json"
            },
            {
              "fileMatch": [
                "behavior_pack/animation_controllers/**/*.json",
                "*BP/animation_controllers/**/*.json",
                "BP_*/animation_controllers/**/*.json",
                "*bp/animation_controllers/**/*.json",
                "bp_*/animation_controllers/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/animationController/main.json"
            },
            {
              "fileMatch": [
                "behavior_pack/animations/**/*.json",
                "*BP/animations/**/*.json",
                "BP_*/animations/**/*.json",
                "*bp/animations/**/*.json",
                "bp_*/animations/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/animation/main.json"
            },
            {
              "fileMatch": [
                "behavior_pack/biomes/**/*.json",
                "*BP/biomes/**/*.json",
                "BP_*/biomes/**/*.json",
                "*bp/biomes/**/*.json",
                "bp_*/biomes/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/biome/main.json"
            },
            {
              "fileMatch": [
                "behavior_pack/blocks/**/*.json",
                "*BP/blocks/**/*.json",
                "BP_*/blocks/**/*.json",
                "*bp/blocks/**/*.json",
                "bp_*/blocks/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/block/main.json"
            },
            {
              "fileMatch": [
                "behavior_pack/cameras/presets/**/*.json",
                "*BP/cameras/presets/**/*.json",
                "BP_*/cameras/presets/**/*.json",
                "*bp/cameras/presets/**/*.json",
                "bp_*/cameras/presets/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/cameraPreset/main.json"
            },
            {
              "fileMatch": [
                "behavior_pack/dialogue/**/*.json",
                "*BP/dialogue/**/*.json",
                "BP_*/dialogue/**/*.json",
                "*bp/dialogue/**/*.json",
                "bp_*/dialogue/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/dialogue/main.json"
            },
            {
              "fileMatch": [
                "behavior_pack/dimensions/**/*.json",
                "*BP/dimensions/**/*.json",
                "BP_*/dimensions/**/*.json",
                "*bp/dimensions/**/*.json",
                "bp_*/dimensions/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/dimension/main.json"
            },
            {
              "fileMatch": [
                "behavior_pack/entities/**/*.json",
                "*BP/entities/**/*.json",
                "BP_*/entities/**/*.json",
                "*bp/entities/**/*.json",
                "bp_*/entities/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/entity/main.json"
            },
            {
              "fileMatch": [
                "behavior_pack/feature_rules/**/*.json",
                "*BP/feature_rules/**/*.json",
                "BP_*/feature_rules/**/*.json",
                "*bp/feature_rules/**/*.json",
                "bp_*/feature_rules/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/featureRule/main.json"
            },
            {
              "fileMatch": [
                "behavior_pack/features/**/*.json",
                "*BP/features/**/*.json",
                "BP_*/features/**/*.json",
                "*bp/features/**/*.json",
                "bp_*/features/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/feature/main.json"
            },
            {
              "fileMatch": [
                "behavior_pack/functions/tick.json",
                "*BP/functions/tick.json",
                "BP_*/functions/tick.json",
                "*bp/functions/tick.json",
                "bp_*/functions/tick.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/tick/main.json"
            },
            {
              "fileMatch": [
                "behavior_pack/item_catalog/crafting_item_catalog.json",
                "*BP/item_catalog/crafting_item_catalog.json",
                "BP_*/item_catalog/crafting_item_catalog.json",
                "*bp/item_catalog/crafting_item_catalog.json",
                "bp_*/item_catalog/crafting_item_catalog.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/craftingItemCatalog/main.json"
            },
            {
              "fileMatch": [
                "behavior_pack/items/**/*.json",
                "*BP/items/**/*.json",
                "BP_*/items/**/*.json",
                "*bp/items/**/*.json",
                "bp_*/items/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/item/main.json"
            },
            {
              "fileMatch": [
                "behavior_pack/loot_tables/**/*.json",
                "*BP/loot_tables/**/*.json",
                "BP_*/loot_tables/**/*.json",
                "*bp/loot_tables/**/*.json",
                "bp_*/loot_tables/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/lootTable/main.json"
            },
            {
              "fileMatch": [
                "behavior_pack/manifest.json",
                "*BP/manifest.json",
                "BP_*/manifest.json",
                "*bp/manifest.json",
                "bp_*/manifest.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/manifest/main.json"
            },
            {
              "fileMatch": [
                "behavior_pack/recipes/**/*.json",
                "*BP/recipes/**/*.json",
                "BP_*/recipes/**/*.json",
                "*bp/recipes/**/*.json",
                "bp_*/recipes/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/recipe/main.json"
            },
            {
              "fileMatch": [
                "behavior_pack/spawn_rules/**/*.json",
                "*BP/spawn_rules/**/*.json",
                "BP_*/spawn_rules/**/*.json",
                "*bp/spawn_rules/**/*.json",
                "bp_*/spawn_rules/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/spawnRule/main.json"
            },
            {
              "fileMatch": [
                "behavior_pack/trading/**/*.json",
                "*BP/trading/**/*.json",
                "BP_*/trading/**/*.json",
                "*bp/trading/**/*.json",
                "bp_*/trading/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/tradeTable/main.json"
            },
            {
              "fileMatch": [
                "behavior_pack/volumes/**/*.json",
                "*BP/volumes/**/*.json",
                "BP_*/volumes/**/*.json",
                "*bp/volumes/**/*.json",
                "bp_*/volumes/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/volume/main.json"
            },
            {
              "fileMatch": [
                "behavior_pack/worldgen/processors/**/*.json",
                "*BP/worldgen/processors/**/*.json",
                "BP_*/worldgen/processors/**/*.json",
                "*bp/worldgen/processors/**/*.json",
                "bp_*/worldgen/processors/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/processorList/main.json"
            },
            {
              "fileMatch": [
                "behavior_pack/worldgen/structure_sets/**/*.json",
                "*BP/worldgen/structure_sets/**/*.json",
                "BP_*/worldgen/structure_sets/**/*.json",
                "*bp/worldgen/structure_sets/**/*.json",
                "bp_*/worldgen/structure_sets/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/structureSet/main.json"
            },
            {
              "fileMatch": [
                "behavior_pack/worldgen/structures/**/*.json",
                "*BP/worldgen/structures/**/*.json",
                "BP_*/worldgen/structures/**/*.json",
                "*bp/worldgen/structures/**/*.json",
                "bp_*/worldgen/structures/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/jigsawStructure/main.json"
            },
            {
              "fileMatch": [
                "behavior_pack/worldgen/template_pools/**/*.json",
                "*BP/worldgen/template_pools/**/*.json",
                "BP_*/worldgen/template_pools/**/*.json",
                "*bp/worldgen/template_pools/**/*.json",
                "bp_*/worldgen/template_pools/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/templatePool/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/animation_controllers/**/*.json",
                "*RP/animation_controllers/**/*.json",
                "RP_*/animation_controllers/**/*.json",
                "*rp/animation_controllers/**/*.json",
                "rp_*/animation_controllers/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/clientAnimationController/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/animations/**/*.json",
                "*RP/animations/**/*.json",
                "RP_*/animations/**/*.json",
                "*rp/animations/**/*.json",
                "rp_*/animations/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/clientAnimation/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/atmospherics/**/*.json",
                "*RP/atmospherics/**/*.json",
                "RP_*/atmospherics/**/*.json",
                "*rp/atmospherics/**/*.json",
                "rp_*/atmospherics/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/atmosphereSettings/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/attachables/**/*.json",
                "*RP/attachables/**/*.json",
                "RP_*/attachables/**/*.json",
                "*rp/attachables/**/*.json",
                "rp_*/attachables/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/attachable/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/biomes_client.json",
                "*RP/biomes_client.json",
                "RP_*/biomes_client.json",
                "*rp/biomes_client.json",
                "rp_*/biomes_client.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/biomesClient/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/biomes/**/*.json",
                "*RP/biomes/**/*.json",
                "RP_*/biomes/**/*.json",
                "*rp/biomes/**/*.json",
                "rp_*/biomes/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/clientBiome/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/block_culling/**/*.json",
                "*RP/block_culling/**/*.json",
                "RP_*/block_culling/**/*.json",
                "*rp/block_culling/**/*.json",
                "rp_*/block_culling/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/blockCulling/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/blocks.json",
                "*RP/blocks.json",
                "RP_*/blocks.json",
                "*rp/blocks.json",
                "rp_*/blocks.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/clientBlock/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/color_grading/**/*.json",
                "*RP/color_grading/**/*.json",
                "RP_*/color_grading/**/*.json",
                "*rp/color_grading/**/*.json",
                "rp_*/color_grading/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/colorGradingSettings/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/entity/**/*.json",
                "*RP/entity/**/*.json",
                "RP_*/entity/**/*.json",
                "*rp/entity/**/*.json",
                "rp_*/entity/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/clientEntity/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/fogs/**/*.json",
                "*RP/fogs/**/*.json",
                "RP_*/fogs/**/*.json",
                "*rp/fogs/**/*.json",
                "rp_*/fogs/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/fog/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/items/**/*.json",
                "*RP/items/**/*.json",
                "RP_*/items/**/*.json",
                "*rp/items/**/*.json",
                "rp_*/items/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/clientItem/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/lighting/**/*.json",
                "*RP/lighting/**/*.json",
                "RP_*/lighting/**/*.json",
                "*rp/lighting/**/*.json",
                "rp_*/lighting/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/lightingSettings/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/manifest.json",
                "*RP/manifest.json",
                "RP_*/manifest.json",
                "*rp/manifest.json",
                "rp_*/manifest.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/manifest/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/materials/**/*.material",
                "*RP/materials/**/*.material",
                "RP_*/materials/**/*.material",
                "*rp/materials/**/*.material",
                "rp_*/materials/**/*.material"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/material/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/models/**/*.json",
                "*RP/models/**/*.json",
                "RP_*/models/**/*.json",
                "*rp/models/**/*.json",
                "rp_*/models/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/geometry/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/particles/**/*.json",
                "*RP/particles/**/*.json",
                "RP_*/particles/**/*.json",
                "*rp/particles/**/*.json",
                "rp_*/particles/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/particle/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/pbr/global.json",
                "*RP/pbr/global.json",
                "RP_*/pbr/global.json",
                "*rp/pbr/global.json",
                "rp_*/pbr/global.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/pbrFallbackSettings/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/point_lights/global.json",
                "*RP/point_lights/global.json",
                "RP_*/point_lights/global.json",
                "*rp/point_lights/global.json",
                "rp_*/point_lights/global.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/pointLightSettings/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/render_controllers/**/*.json",
                "*RP/render_controllers/**/*.json",
                "RP_*/render_controllers/**/*.json",
                "*rp/render_controllers/**/*.json",
                "rp_*/render_controllers/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/renderController/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/shadows/global.json",
                "*RP/shadows/global.json",
                "RP_*/shadows/global.json",
                "*rp/shadows/global.json",
                "rp_*/shadows/global.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/shadowSettings/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/sounds.json",
                "*RP/sounds.json",
                "RP_*/sounds.json",
                "*rp/sounds.json",
                "rp_*/sounds.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/clientSound/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/sounds/music_definitions.json",
                "*RP/sounds/music_definitions.json",
                "RP_*/sounds/music_definitions.json",
                "*rp/sounds/music_definitions.json",
                "rp_*/sounds/music_definitions.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/musicDefinition/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/sounds/sound_definitions.json",
                "*RP/sounds/sound_definitions.json",
                "RP_*/sounds/sound_definitions.json",
                "*rp/sounds/sound_definitions.json",
                "rp_*/sounds/sound_definitions.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/soundDefinition/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/splashes.json",
                "*RP/splashes.json",
                "RP_*/splashes.json",
                "*rp/splashes.json",
                "rp_*/splashes.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/splashes/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/textures/**/*.texture_set.json",
                "*RP/textures/**/*.texture_set.json",
                "RP_*/textures/**/*.texture_set.json",
                "*rp/textures/**/*.texture_set.json",
                "rp_*/textures/**/*.texture_set.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/textureSet/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/textures/flipbook_textures.json",
                "*RP/textures/flipbook_textures.json",
                "RP_*/textures/flipbook_textures.json",
                "*rp/textures/flipbook_textures.json",
                "rp_*/textures/flipbook_textures.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/flipbookTexture/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/textures/item_texture.json",
                "*RP/textures/item_texture.json",
                "RP_*/textures/item_texture.json",
                "*rp/textures/item_texture.json",
                "rp_*/textures/item_texture.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/itemTexture/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/textures/terrain_texture.json",
                "*RP/textures/terrain_texture.json",
                "RP_*/textures/terrain_texture.json",
                "*rp/textures/terrain_texture.json",
                "rp_*/textures/terrain_texture.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/terrainTexture/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/ui/**/*.json",
                "*RP/ui/**/*.json",
                "RP_*/ui/**/*.json",
                "*rp/ui/**/*.json",
                "rp_*/ui/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/ui/main.json"
            },
            {
              "fileMatch": [
                "resource_pack/water/**/*.json",
                "*RP/water/**/*.json",
                "RP_*/water/**/*.json",
                "*rp/water/**/*.json",
                "rp_*/water/**/*.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/waterSettings/main.json"
            },
            {
              "fileMatch": [
                "skin_pack/skins.json"
              ],
              "url": "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/skins/main.json"
            }
          ]
        }
      }
    }
  }
}
```
