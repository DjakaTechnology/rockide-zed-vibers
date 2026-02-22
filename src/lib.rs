use std::fs;
use zed::serde_json::{self, json};
use zed_extension_api::{self as zed, LanguageServerId, Result};

struct RockideExtension {
    cached_binary_path: Option<String>,
}

impl RockideExtension {
    fn language_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        if let Some(path) = worktree.which("rockide") {
            return Ok(path);
        }

        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
            "rockide/language-server",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = zed::current_platform();
        let os = match platform {
            zed::Os::Mac => "darwin",
            zed::Os::Linux => "linux",
            zed::Os::Windows => "windows",
        };
        let cpu = match arch {
            zed::Architecture::Aarch64 => "arm64",
            zed::Architecture::X8664 | zed::Architecture::X86 => "amd64",
        };

        let asset_name = format!("rockide_{version}_{os}_{cpu}.tar.gz", version = release.version);

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {:?}", asset_name))?;

        let version_dir = format!("rockide-{}", release.version);
        let binary_name = match platform {
            zed::Os::Windows => "rockide.exe",
            _ => "rockide",
        };
        let binary_path = format!("{version_dir}/{binary_name}");

        if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &asset.download_url,
                &version_dir,
                zed::DownloadedFileType::GzipTar,
            )
            .map_err(|e| format!("failed to download file: {e}"))?;

            let entries =
                fs::read_dir(".").map_err(|e| format!("failed to list working directory {e}"))?;
            for entry in entries {
                let entry = entry.map_err(|e| format!("failed to load directory entry {e}"))?;
                if entry.file_name().to_str() != Some(&version_dir) {
                    fs::remove_dir_all(entry.path()).ok();
                }
            }
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

impl zed::Extension for RockideExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        Ok(zed::Command {
            command: self.language_server_binary_path(language_server_id, worktree)?,
            args: vec![],
            env: Default::default(),
        })
    }

    fn language_server_additional_workspace_configuration(
        &mut self,
        _language_server_id: &LanguageServerId,
        target_language_server_id: &LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        if target_language_server_id.as_ref() != "json-language-server" {
            return Ok(None);
        }

        let base = "https://raw.githubusercontent.com/rockide/editor-packages/rockide/packages/minecraftBedrock/schema";

        Ok(Some(json!({
            "json": {
                "schemas": schemas(base)
            }
        })))
    }
}

fn bp(path: &str) -> Vec<String> {
    ["behavior_pack", "*BP", "BP_*", "*bp", "bp_*"]
        .iter()
        .map(|p| format!("{p}/{path}"))
        .collect()
}

fn rp(path: &str) -> Vec<String> {
    ["resource_pack", "*RP", "RP_*", "*rp", "rp_*"]
        .iter()
        .map(|p| format!("{p}/{path}"))
        .collect()
}

fn schema(base: &str, name: &str, file_match: Vec<String>) -> serde_json::Value {
    json!({
        "fileMatch": file_match,
        "url": format!("{base}/{name}/main.json")
    })
}

fn schemas(base: &str) -> Vec<serde_json::Value> {
    vec![
        // Behavior Pack schemas
        schema(base, "aimAssistCategories", bp("aim_assist/categories/**/*.json")),
        schema(base, "aimAssistPreset", bp("aim_assist/presets/**/*.json")),
        schema(base, "animationController", bp("animation_controllers/**/*.json")),
        schema(base, "animation", bp("animations/**/*.json")),
        schema(base, "biome", bp("biomes/**/*.json")),
        schema(base, "block", bp("blocks/**/*.json")),
        schema(base, "cameraPreset", bp("cameras/presets/**/*.json")),
        schema(base, "dialogue", bp("dialogue/**/*.json")),
        schema(base, "dimension", bp("dimensions/**/*.json")),
        schema(base, "entity", bp("entities/**/*.json")),
        schema(base, "featureRule", bp("feature_rules/**/*.json")),
        schema(base, "feature", bp("features/**/*.json")),
        schema(base, "tick", bp("functions/tick.json")),
        schema(base, "craftingItemCatalog", bp("item_catalog/crafting_item_catalog.json")),
        schema(base, "item", bp("items/**/*.json")),
        schema(base, "lootTable", bp("loot_tables/**/*.json")),
        schema(base, "manifest", bp("manifest.json")),
        schema(base, "recipe", bp("recipes/**/*.json")),
        schema(base, "spawnRule", bp("spawn_rules/**/*.json")),
        schema(base, "tradeTable", bp("trading/**/*.json")),
        schema(base, "volume", bp("volumes/**/*.json")),
        schema(base, "processorList", bp("worldgen/processors/**/*.json")),
        schema(base, "structureSet", bp("worldgen/structure_sets/**/*.json")),
        schema(base, "jigsawStructure", bp("worldgen/structures/**/*.json")),
        schema(base, "templatePool", bp("worldgen/template_pools/**/*.json")),
        // Resource Pack schemas
        schema(base, "clientAnimationController", rp("animation_controllers/**/*.json")),
        schema(base, "clientAnimation", rp("animations/**/*.json")),
        schema(base, "atmosphereSettings", rp("atmospherics/**/*.json")),
        schema(base, "attachable", rp("attachables/**/*.json")),
        schema(base, "biomesClient", rp("biomes_client.json")),
        schema(base, "clientBiome", rp("biomes/**/*.json")),
        schema(base, "blockCulling", rp("block_culling/**/*.json")),
        schema(base, "clientBlock", rp("blocks.json")),
        schema(base, "colorGradingSettings", rp("color_grading/**/*.json")),
        schema(base, "clientEntity", rp("entity/**/*.json")),
        schema(base, "fog", rp("fogs/**/*.json")),
        schema(base, "clientItem", rp("items/**/*.json")),
        schema(base, "lightingSettings", rp("lighting/**/*.json")),
        schema(base, "manifest", rp("manifest.json")),
        schema(base, "material", rp("materials/**/*.material")),
        schema(base, "geometry", rp("models/**/*.json")),
        schema(base, "particle", rp("particles/**/*.json")),
        schema(base, "pbrFallbackSettings", rp("pbr/global.json")),
        schema(base, "pointLightSettings", rp("point_lights/global.json")),
        schema(base, "renderController", rp("render_controllers/**/*.json")),
        schema(base, "shadowSettings", rp("shadows/global.json")),
        schema(base, "clientSound", rp("sounds.json")),
        schema(base, "musicDefinition", rp("sounds/music_definitions.json")),
        schema(base, "soundDefinition", rp("sounds/sound_definitions.json")),
        schema(base, "splashes", rp("splashes.json")),
        schema(base, "textureSet", rp("textures/**/*.texture_set.json")),
        schema(base, "flipbookTexture", rp("textures/flipbook_textures.json")),
        schema(base, "itemTexture", rp("textures/item_texture.json")),
        schema(base, "terrainTexture", rp("textures/terrain_texture.json")),
        schema(base, "ui", rp("ui/**/*.json")),
        schema(base, "waterSettings", rp("water/**/*.json")),
        // Skin Pack
        schema(base, "skins", vec!["skin_pack/skins.json".to_string()]),
    ]
}

zed::register_extension!(RockideExtension);
