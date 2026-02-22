# Rockide for Zed — Implementation Notes

## Overview

Port of [rockide-vscode](https://github.com/rockide/rockide-vscode) to Zed Editor.
Provides Minecraft Bedrock Edition addon development support using the
[Rockide language server](https://github.com/rockide/language-server) (Go binary, stdio).

---

## What Was Built

### Project Structure

```
rockide-zed-vibers/
├── extension.toml              # Extension manifest
├── Cargo.toml                  # Rust crate config (cdylib → Wasm)
├── src/
│   └── lib.rs                  # Extension impl: binary download + schema injection
├── languages/
│   └── lang/
│       └── config.toml         # Minecraft .lang language definition
└── docs/
    └── PLAN.md                 # This file
```

### Extension Manifest (`extension.toml`)

- Registers the `rockide` language server for **JSON**, **JSONC**, and **Minecraft Lang**
- References the existing [`tree-sitter-lang`](https://github.com/rockide/tree-sitter-lang) grammar at `fca23509c5fe8c74e03a25b42ec72405808bd3d3`
- Language IDs: `json`, `jsonc`, `rockide-lang`

### Language Server Binary Management (`src/lib.rs`)

Resolution order:
1. Check PATH for `rockide` binary (`worktree.which`)
2. Check cached binary path from previous download
3. Download from GitHub releases (`rockide/language-server`)

Platform mapping:
- OS: Mac → `darwin`, Linux → `linux`, Windows → `windows`
- Arch: Aarch64 → `arm64`, X8664/X86 → `amd64`
- Asset: `rockide_{version}_{os}_{arch}.tar.gz` (all platforms use GzipTar)
- Binary: `rockide` / `rockide.exe` (Windows)

Old version directories are cleaned up after downloading a new version.

### JSON Schema Injection (57 schemas)

**Approach**: Programmatic injection via `language_server_additional_workspace_configuration`.

Instead of requiring users to manually configure schemas in `.zed/settings.json`,
the extension injects all 57 schema mappings directly into Zed's built-in
`json-language-server` at runtime. This works because the Zed Extension API
allows one language server's extension to provide workspace configuration to
another language server.

The method checks `target_language_server_id == "json-language-server"` and returns
a `json.schemas` array with all fileMatch → URL mappings.

Schema source: [`rockide/editor-packages`](https://github.com/rockide/editor-packages)
(branch `rockide`), URL pattern:
```
https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema/{name}/main.json
```

**Caching behavior**: Zed's json-language-server caches fetched schemas in memory
with no TTL. Schemas persist for the lifetime of the LSP process. To refetch
updated schemas, users must restart the language server via
`editor: restart language server` in the command palette.

### Minecraft Lang Support

- Language config: `languages/lang/config.toml`
- Grammar: external [`tree-sitter-lang`](https://github.com/rockide/tree-sitter-lang) repo
  (Zed fetches and compiles it automatically)
- File extension: `.lang`
- Line comments: `## `
- Highlights: provided by the grammar repo's `queries/highlights.scm`

---

## What Was Deferred

| Feature | Reason |
|---|---|
| Molang syntax highlighting | `tree-sitter-molang` grammar repo doesn't exist yet |
| `.material` file registration as JSON | Needs investigation on Zed's support for adding suffixes to built-in languages |
| `projectPaths` initialization options | Not yet wired up; users can configure via `lsp.rockide.initialization_options` in settings |
| Offline schema support | Schemas currently fetched from GitHub URLs; bundling or local caching TBD |
| Semantic token color configuration | Documented in README as user-configured settings (28 Minecraft color tokens) |

---

## Architecture Decisions

### Schema injection vs user configuration

**Decision**: Programmatic injection via `language_server_additional_workspace_configuration`

**Rationale**: The original plan required users to copy a ~600-line JSON block into
`.zed/settings.json`. This was impractical. The Zed Extension API allows extensions
to inject workspace configuration into other language servers, so we inject the
schema mappings directly into `json-language-server`. Zero user configuration needed.

### Grammar hosting

**Decision**: Use existing `rockide/tree-sitter-lang` repo, skip Molang

**Rationale**: Zed requires grammars in separate Git repos (referenced by commit SHA
in `extension.toml`). The `.lang` grammar already exists at `rockide/tree-sitter-lang`.
Molang would need a new `tree-sitter-molang` repo — deferred until created.

### Schema source

**Decision**: `rockide/editor-packages` (branch `rockide`) instead of `bridge-core/editor-packages`

**Rationale**: The rockide fork/branch contains the schemas maintained for the rockide
ecosystem. Using raw GitHub URLs means schemas are always the latest published version
(fetched once per LSP session).

---

## Dependencies

| Crate | Version | Purpose |
|---|---|---|
| `zed_extension_api` | 0.7.0 | Zed extension trait, LSP management, platform detection |
| `serde_json` | 1.0 | JSON schema construction for workspace config injection |

---

## Testing

1. `cargo build --target wasm32-wasip1` — verify Wasm compiles
2. In Zed → Extensions → "Install Dev Extension" → select project directory
3. Open a Minecraft Bedrock addon project:
   - Verify language server binary downloads automatically
   - Verify completions/go-to-definition in JSON files
   - Verify `.lang` files get syntax highlighting
   - Verify JSON schema validation diagnostics appear
