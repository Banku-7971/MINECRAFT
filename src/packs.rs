//! Pure-Rust loader for real Minecraft Resource/Texture Packs (.zip),
//! OptiFine/Iris Shader Packs (.zip), Datapacks (data/*/recipes/*.json),
//! and Modrinth/CurseForge Modpacks (.mrpack / manifest.json).
//! Zero external dependencies — compiles identically for Desktop and WebAssembly.

use crate::craft;
use crate::items;
use crate::render::{ShaderConfig, Textures};
use crate::sound::Sfx;
use crate::world;
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackKind {
    ResourcePack,
    ShaderPack,
    DataPack,
    ModPack,
}

impl PackKind {
    pub fn label(&self) -> &'static str {
        match self {
            PackKind::ResourcePack => "RESOURCE",
            PackKind::ShaderPack => "SHADER",
            PackKind::DataPack => "DATAPACK",
            PackKind::ModPack => "MODPACK",
        }
    }
}

#[derive(Clone, Debug)]
pub struct LoadedPackInfo {
    pub name: String,
    pub kind: PackKind,
    pub description: String,
    pub textures_replaced: usize,
    pub recipes_added: usize,
    pub mod_files: Vec<(String, String)>, // (filename, download_url)
}

#[derive(Clone)]
pub struct PackRegistry {
    pub packs: Vec<LoadedPackInfo>,
    pub active_shader: ShaderConfig,
    pub shader_preset_idx: usize,
    pub active_textures: Textures,
    pub mod_downloads: Vec<(String, String)>,
    pub custom_sounds: Vec<(u8, Vec<u8>)>,
    pub sounds_version: u32,
}

impl PackRegistry {
    fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let (shader_preset_idx, active_shader) = (5, ShaderConfig::preset(5));
        #[cfg(target_arch = "wasm32")]
        let (shader_preset_idx, active_shader) = (0, ShaderConfig::vanilla());
        Self {
            packs: Vec::new(),
            active_shader,
            shader_preset_idx,
            active_textures: Textures::new(),
            mod_downloads: Vec::new(),
            custom_sounds: Vec::new(),
            sounds_version: 0,
        }
    }
}

static PACK_REGISTRY: OnceLock<RwLock<PackRegistry>> = OnceLock::new();

fn registry() -> &'static RwLock<PackRegistry> {
    PACK_REGISTRY.get_or_init(|| RwLock::new(PackRegistry::new()))
}

pub fn active_shader() -> ShaderConfig {
    registry()
        .read()
        .map(|r| r.active_shader.clone())
        .unwrap_or_else(|_| ShaderConfig::vanilla())
}

pub fn cycle_shader_preset() -> ShaderConfig {
    if let Ok(mut r) = registry().write() {
        r.shader_preset_idx = (r.shader_preset_idx + 1) % 6;
        r.active_shader = ShaderConfig::preset(r.shader_preset_idx);
        return r.active_shader.clone();
    }
    ShaderConfig::vanilla()
}

pub fn set_shader_preset(idx: usize) -> ShaderConfig {
    if let Ok(mut r) = registry().write() {
        r.shader_preset_idx = idx % 6;
        r.active_shader = ShaderConfig::preset(r.shader_preset_idx);
        return r.active_shader.clone();
    }
    ShaderConfig::vanilla()
}

#[allow(dead_code)]
pub fn set_custom_shader(cfg: ShaderConfig) {
    if let Ok(mut r) = registry().write() {
        r.active_shader = cfg;
    }
}

pub fn loaded_packs() -> Vec<LoadedPackInfo> {
    registry()
        .read()
        .map(|r| r.packs.clone())
        .unwrap_or_default()
}

pub fn pack_count() -> usize {
    registry().read().map(|r| r.packs.len()).unwrap_or(0)
}

pub fn reset_all_packs() {
    craft::clear_custom_recipes();
    if let Ok(mut r) = registry().write() {
        r.packs.clear();
        r.mod_downloads.clear();
        r.custom_sounds.clear();
        r.sounds_version = r.sounds_version.wrapping_add(1);
        r.active_textures.reset();
        r.active_shader = ShaderConfig::preset(r.shader_preset_idx);
        Textures::set_active(&r.active_textures);
    }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub fn custom_sound_version() -> u32 {
    registry().read().map(|r| r.sounds_version).unwrap_or(0)
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub fn custom_sound_count() -> usize {
    registry().read().map(|r| r.custom_sounds.len()).unwrap_or(0)
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub fn custom_sound_at(idx: usize) -> Option<(u8, *const u8, usize)> {
    registry().read().ok().and_then(|r| {
        r.custom_sounds.get(idx).map(|(sfx, bytes)| (*sfx, bytes.as_ptr(), bytes.len()))
    })
}

pub fn custom_sound_bytes(sfx_id: u8) -> Option<Vec<u8>> {
    registry().read().ok().and_then(|r| {
        r.custom_sounds.iter().find(|(id, _)| *id == sfx_id).map(|(_, b)| b.clone())
    })
}

/// Summary string returned to UI / JS when a pack is loaded.
pub fn load_pack_bytes(filename: &str, bytes: &[u8]) -> Result<LoadedPackInfo, String> {
    let lower_name = filename.to_ascii_lowercase();
    if lower_name.ends_with(".json") {
        let text = core::str::from_utf8(bytes).map_err(|_| String::from("Invalid UTF-8 JSON"))?;
        if let Some(info) = try_load_standalone_json(filename, text) {
            register_loaded_pack(&info, None);
            return Ok(info);
        }
        return Err(String::from("Unrecognized JSON pack/recipe format"));
    }

    let entries = read_zip_entries(bytes)?;
    if entries.is_empty() {
        return Err(String::from("Archive contains no readable files"));
    }

    let mut is_mrpack = false;
    let mut is_curseforge = false;
    let mut has_shaders = false;
    let mut has_textures = false;
    let mut has_sounds = false;
    let mut has_recipes = false;
    let mut description = String::new();
    let mut pack_title = filename
        .rsplit(&['/', '\\'][..])
        .next()
        .unwrap_or(filename)
        .to_string();

    let mut grass_tint: Option<[f32; 3]> = None;
    let mut foliage_tint: Option<[f32; 3]> = None;
    let mut mod_files: Vec<(String, String)> = Vec::new();
    let mut sub_zips: Vec<(String, Vec<u8>)> = Vec::new();

    // Pre-scan metadata, colormaps, and classification
    for (name, data) in &entries {
        let ln = name.to_ascii_lowercase();
        if ln == "modrinth.index.json" || ln.ends_with("/modrinth.index.json") {
            is_mrpack = true;
            if let Ok(txt) = core::str::from_utf8(data) {
                if let Some(j) = parse_json(txt) {
                    if let Some(n) = j.get("name").and_then(|v| v.as_str()) {
                        pack_title = n.to_string();
                    }
                    if let Some(s) = j.get("summary").and_then(|v| v.as_str()) {
                        description = s.to_string();
                    }
                    if let Some(arr) = j.get("files").and_then(|v| v.as_array()) {
                        for f in arr {
                            let fpath = f.get("path").and_then(|v| v.as_str()).unwrap_or("mod.jar");
                            let dl = f
                                .get("downloads")
                                .and_then(|v| v.as_array())
                                .and_then(|a| a.first())
                                .and_then(|v| v.as_str())
                                .unwrap_or("");
                            if !dl.is_empty() {
                                mod_files.push((fpath.to_string(), dl.to_string()));
                            }
                        }
                    }
                }
            }
        } else if ln == "manifest.json" || ln.ends_with("/manifest.json") {
            if let Ok(txt) = core::str::from_utf8(data) {
                if let Some(j) = parse_json(txt) {
                    if j.get("files").is_some() || j.get("minecraft").is_some() {
                        is_curseforge = true;
                        if let Some(n) = j.get("name").and_then(|v| v.as_str()) {
                            pack_title = n.to_string();
                        }
                        if let Some(v) = j.get("version").and_then(|v| v.as_str()) {
                            description = format!("CurseForge v{}", v);
                        }
                        if let Some(arr) = j.get("files").and_then(|v| v.as_array()) {
                            for f in arr {
                                let pid = f.get("projectID").and_then(|v| v.as_i64()).unwrap_or(0);
                                let fid = f.get("fileID").and_then(|v| v.as_i64()).unwrap_or(0);
                                if pid > 0 && fid > 0 {
                                    mod_files.push((
                                        format!("curseforge:{}:{}", pid, fid),
                                        format!("https://www.curseforge.com/api/v1/mods/{}/files/{}/download", pid, fid),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        } else if ln.ends_with("pack.mcmeta") {
            if let Ok(txt) = core::str::from_utf8(data) {
                if let Some(j) = parse_json(txt) {
                    if let Some(p) = j.get("pack") {
                        if let Some(d) = p.get("description") {
                            description = json_text_to_string(d);
                        }
                    }
                }
            }
        } else if ln.ends_with("colormap/grass.png") {
            if let Ok((w, h, rgba)) = decode_png(data) {
                grass_tint = sample_center_rgb(w, h, &rgba);
            }
        } else if ln.ends_with("colormap/foliage.png") {
            if let Ok((w, h, rgba)) = decode_png(data) {
                foliage_tint = sample_center_rgb(w, h, &rgba);
            }
        } else if ln.contains("/shaders/") || ln.starts_with("shaders/") {
            if ln.ends_with(".fsh") || ln.ends_with(".vsh") || ln.ends_with("shaders.properties") {
                has_shaders = true;
            }
        } else if ln.contains("textures/block") || ln.contains("textures/item") {
            if ln.ends_with(".png") {
                has_textures = true;
            }
        } else if ln.contains("sounds/") && (ln.ends_with(".ogg") || ln.ends_with(".wav")) {
            has_sounds = true;
        } else if (ln.contains("/recipes/") || ln.contains("/recipe/")) && ln.ends_with(".json") {
            has_recipes = true;
        } else if (ln.contains("overrides/") || ln.contains("resourcepacks/") || ln.contains("shaderpacks/"))
            && ln.ends_with(".zip")
        {
            sub_zips.push((name.clone(), data.clone()));
        }
    }

    let mut textures_replaced = 0usize;
    let mut recipes_added = 0usize;
    let mut shader_cfg: Option<ShaderConfig> = None;

    // 1) Parse shader pack properties & GLSL defines
    if has_shaders {
        let cfg = parse_shader_pack(&pack_title, &entries);
        shader_cfg = Some(cfg);
    }

    // 2) Apply textures (blocks + items) into active atlas
    if has_textures {
        let mut tex = registry()
            .read()
            .map(|r| r.active_textures.clone())
            .unwrap_or_else(|_| Textures::new());

        for (name, data) in &entries {
            let ln = name.to_ascii_lowercase().replace('\\', "/");
            if !ln.ends_with(".png") {
                continue;
            }
            if let Some(stem) = extract_texture_stem(&ln, "textures/block/")
                .or_else(|| extract_texture_stem(&ln, "textures/blocks/"))
            {
                if let Some(slots) = map_block_texture_slots(stem) {
                    if let Ok((w, h, rgba)) = decode_png(data) {
                        let mut tile = downsample_to_16x16(w, h, &rgba);
                        if is_tinted_grass_stem(stem) {
                            let tint = grass_tint.unwrap_or([0.56, 0.80, 0.36]);
                            apply_grayscale_tint(&mut tile, tint);
                        } else if is_tinted_foliage_stem(stem) {
                            let tint = foliage_tint.unwrap_or([0.46, 0.76, 0.30]);
                            apply_grayscale_tint(&mut tile, tint);
                        }
                        for &slot in slots {
                            tex.set_block_tile(slot, &tile);
                        }
                        textures_replaced += 1;
                    }
                }
            }
            if let Some(stem) = extract_texture_stem(&ln, "textures/item/")
                .or_else(|| extract_texture_stem(&ln, "textures/items/"))
            {
                if let Some(slot) = map_item_texture_slot(stem) {
                    if let Ok((w, h, rgba)) = decode_png(data) {
                        let tile = downsample_to_16x16(w, h, &rgba);
                        tex.set_item_tile(slot, &tile);
                        textures_replaced += 1;
                    }
                }
            }
        }

        if textures_replaced > 0 {
            if let Ok(mut r) = registry().write() {
                r.active_textures = tex.clone();
                Textures::set_active(&r.active_textures);
            }
        }
    }

    // 2b) Extract Minecraft resource-pack sounds (.ogg / .wav under sounds/)
    let mut sounds_added = 0usize;
    if has_sounds {
        let mut loaded_sfx: Vec<(u8, Vec<u8>)> = Vec::new();
        for (name, data) in &entries {
            let ln = name.to_ascii_lowercase().replace('\\', "/");
            if !(ln.ends_with(".ogg") || ln.ends_with(".wav")) {
                continue;
            }
            if let Some(sfx) = map_minecraft_sound_sfx(&ln) {
                let id = sfx as u8;
                if !loaded_sfx.iter().any(|(existing, _)| *existing == id) && !data.is_empty() {
                    loaded_sfx.push((id, data.clone()));
                    sounds_added += 1;
                }
            }
        }
        if !loaded_sfx.is_empty() {
            if let Ok(mut r) = registry().write() {
                for (id, bytes) in loaded_sfx {
                    r.custom_sounds.retain(|(k, _)| *k != id);
                    r.custom_sounds.push((id, bytes));
                }
                r.sounds_version = r.sounds_version.wrapping_add(1);
            }
        }
    }

    // 3) Parse datapack recipes (data/*/recipes/*.json or data/*/recipe/*.json)
    if has_recipes {
        for (name, data) in &entries {
            let ln = name.to_ascii_lowercase().replace('\\', "/");
            if (ln.contains("/recipes/") || ln.contains("/recipe/")) && ln.ends_with(".json") {
                if let Ok(txt) = core::str::from_utf8(data) {
                    if parse_and_register_recipe(txt) {
                        recipes_added += 1;
                    }
                }
            }
        }
    }

    // 4) Load nested resource/shader zips inside modpack overrides
    for (sub_name, sub_data) in sub_zips {
        if let Ok(sub_info) = load_pack_bytes(&sub_name, &sub_data) {
            textures_replaced += sub_info.textures_replaced;
            recipes_added += sub_info.recipes_added;
        }
    }

    let kind = if is_mrpack || is_curseforge {
        PackKind::ModPack
    } else if has_shaders && !has_textures && !has_sounds {
        PackKind::ShaderPack
    } else if has_recipes && !has_textures && !has_shaders && !has_sounds {
        PackKind::DataPack
    } else {
        PackKind::ResourcePack
    };

    if description.is_empty() {
        description = match kind {
            PackKind::ResourcePack => {
                if sounds_added > 0 && textures_replaced > 0 {
                    format!("{} textures, {} sounds applied", textures_replaced, sounds_added)
                } else if sounds_added > 0 {
                    format!("{} sounds applied", sounds_added)
                } else {
                    format!("{} textures applied", textures_replaced)
                }
            }
            PackKind::ShaderPack => String::from("OptiFine/Iris shader config active"),
            PackKind::DataPack => format!("{} custom recipes registered", recipes_added),
            PackKind::ModPack => format!("{} mods indexed", mod_files.len()),
        };
    }

    let info = LoadedPackInfo {
        name: pack_title,
        kind,
        description,
        textures_replaced,
        recipes_added,
        mod_files,
    };

    register_loaded_pack(&info, shader_cfg);
    Ok(info)
}

fn register_loaded_pack(info: &LoadedPackInfo, shader_cfg: Option<ShaderConfig>) {
    if let Ok(mut r) = registry().write() {
        if let Some(cfg) = shader_cfg {
            r.active_shader = cfg;
        }
        for (fname, url) in &info.mod_files {
            if !r.mod_downloads.iter().any(|(_, u)| u == url) {
                r.mod_downloads.push((fname.clone(), url.clone()));
            }
        }
        r.packs.retain(|p| p.name != info.name || p.kind != info.kind);
        r.packs.push(info.clone());
    }
}

fn try_load_standalone_json(filename: &str, txt: &str) -> Option<LoadedPackInfo> {
    let j = parse_json(txt)?;
    // Standalone Modrinth index
    if j.get("files").is_some() && j.get("formatVersion").is_some() {
        let name = j
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(filename)
            .to_string();
        let desc = j
            .get("summary")
            .and_then(|v| v.as_str())
            .unwrap_or("Modrinth Modpack Index")
            .to_string();
        let mut mod_files = Vec::new();
        if let Some(arr) = j.get("files").and_then(|v| v.as_array()) {
            for f in arr {
                let p = f.get("path").and_then(|v| v.as_str()).unwrap_or("mod.jar");
                if let Some(url) = f
                    .get("downloads")
                    .and_then(|v| v.as_array())
                    .and_then(|a| a.first())
                    .and_then(|v| v.as_str())
                {
                    mod_files.push((p.to_string(), url.to_string()));
                }
            }
        }
        return Some(LoadedPackInfo {
            name,
            kind: PackKind::ModPack,
            description: desc,
            textures_replaced: 0,
            recipes_added: 0,
            mod_files,
        });
    }
    // Standalone datapack recipe JSON
    if parse_and_register_recipe(txt) {
        return Some(LoadedPackInfo {
            name: filename.to_string(),
            kind: PackKind::DataPack,
            description: String::from("Custom crafting/smelting recipe"),
            textures_replaced: 0,
            recipes_added: 1,
            mod_files: Vec::new(),
        });
    }
    None
}

#[cfg(not(target_arch = "wasm32"))]
pub fn scan_packs_dir(dir: &std::path::Path) -> Vec<LoadedPackInfo> {
    let mut loaded = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else {
        return loaded;
    };
    let mut paths: Vec<std::path::PathBuf> = rd
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.extension()
                .and_then(|s| s.to_str())
                .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "zip" | "mrpack" | "json"))
                .unwrap_or(false)
        })
        .collect();
    paths.sort();
    for p in paths {
        if let Ok(bytes) = std::fs::read(&p) {
            let name = p
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("pack.zip");
            if let Ok(info) = load_pack_bytes(name, &bytes) {
                loaded.push(info);
            }
        }
    }
    loaded
}

// =========================================================================
// OptiFine / Iris Shader Pack Parser -> ShaderConfig
// =========================================================================

fn parse_shader_pack(title: &str, entries: &[(String, Vec<u8>)]) -> ShaderConfig {
    let lt = title.to_ascii_lowercase();
    let mut cfg = if lt.contains("bsl") {
        ShaderConfig::preset(1)
    } else if lt.contains("complementary") || lt.contains("reimagined") || lt.contains("unbound") {
        ShaderConfig::preset(2)
    } else if lt.contains("sildur") || lt.contains("seus") {
        ShaderConfig::preset(3)
    } else if lt.contains("nostalgia") || lt.contains("vanilla") || lt.contains("chocapic") {
        ShaderConfig::preset(4)
    } else {
        ShaderConfig {
            name: String::new(),
            sun_shadows: true,
            water_reflect: true,
            warm_torch: 1.25,
            saturation: 1.12,
            contrast: 1.06,
            brightness: 1.03,
            fog_density: 1.15,
            sky_warmth: 0.10,
            tonemap_aces: true,
        }
    };
    let short_name = title
        .trim_end_matches(".zip")
        .trim_end_matches(".ZIP")
        .chars()
        .take(20)
        .collect::<String>()
        .to_ascii_uppercase();
    cfg.name = short_name;

    for (name, data) in entries {
        let ln = name.to_ascii_lowercase();
        if !ln.ends_with(".fsh") && !ln.ends_with(".vsh") && !ln.ends_with(".properties") && !ln.ends_with(".glsl") {
            continue;
        }
        let Ok(txt) = core::str::from_utf8(data) else { continue };
        for raw_line in txt.lines() {
            let line = raw_line.trim();
            if let Some(rest) = line.strip_prefix("#define ") {
                let mut parts = rest.split_whitespace();
                let key = parts.next().unwrap_or("");
                let val = parts.next().unwrap_or("").parse::<f32>().ok();
                let ku = key.to_ascii_uppercase();
                if ku.contains("SHADOW") && (ku.contains("OFF") || val == Some(0.0)) {
                    cfg.sun_shadows = false;
                } else if ku.contains("WATER_REFLECT") && val == Some(0.0) {
                    cfg.water_reflect = false;
                } else if ku == "SATURATION" {
                    if let Some(v) = val {
                        cfg.saturation = v.clamp(0.5, 1.8);
                    }
                } else if ku == "CONTRAST" {
                    if let Some(v) = val {
                        cfg.contrast = v.clamp(0.7, 1.5);
                    }
                } else if ku == "BRIGHTNESS" || ku == "EXPOSURE" {
                    if let Some(v) = val {
                        cfg.brightness = v.clamp(0.6, 1.6);
                    }
                } else if ku.contains("FOG_DENSITY") {
                    if let Some(v) = val {
                        cfg.fog_density = v.clamp(0.4, 2.5);
                    }
                } else if ku.contains("ACES") || ku.contains("TONEMAP") {
                    if val == Some(0.0) {
                        cfg.tonemap_aces = false;
                    } else {
                        cfg.tonemap_aces = true;
                    }
                }
            }
        }
    }
    cfg
}

// =========================================================================
// Texture Mapping (Minecraft 1.8..1.21 -> 144 Block & 128 Item Atlas Slots)
// =========================================================================

fn extract_texture_stem<'a>(path: &'a str, marker: &str) -> Option<&'a str> {
    let idx = path.find(marker)?;
    let rest = &path[idx + marker.len()..];
    // Ignore nested subdirectories unless needed, take filename stem
    let fname = rest.rsplit('/').next().unwrap_or(rest);
    fname.strip_suffix(".png")
}

fn is_tinted_grass_stem(stem: &str) -> bool {
    matches!(stem, "grass_block_top" | "grass_top" | "tall_grass" | "tallgrass" | "short_grass" | "grass")
}

fn is_tinted_foliage_stem(stem: &str) -> bool {
    matches!(
        stem,
        "oak_leaves" | "leaves_oak" | "spruce_leaves" | "leaves_spruce" | "birch_leaves" | "leaves_birch" | "jungle_leaves" | "dark_oak_leaves" | "vine"
    )
}

fn sample_center_rgb(w: usize, h: usize, rgba: &[u8]) -> Option<[f32; 3]> {
    if w == 0 || h == 0 || rgba.len() < w * h * 4 {
        return None;
    }
    let cx = w / 2;
    let cy = h / 2;
    let i = (cy * w + cx) * 4;
    Some([
        (rgba[i] as f32 / 255.0).clamp(0.25, 1.0),
        (rgba[i + 1] as f32 / 255.0).clamp(0.25, 1.0),
        (rgba[i + 2] as f32 / 255.0).clamp(0.20, 1.0),
    ])
}

fn apply_grayscale_tint(tile: &mut [u8; 1024], tint: [f32; 3]) {
    // Check if the texture is predominantly neutral grayscale (standard Minecraft colormap mask)
    let mut sum_diff = 0u32;
    let mut opaque = 0u32;
    for px in tile.chunks_exact(4) {
        if px[3] > 20 {
            let maxc = px[0].max(px[1]).max(px[2]);
            let minc = px[0].min(px[1]).min(px[2]);
            sum_diff += (maxc - minc) as u32;
            opaque += 1;
        }
    }
    if opaque > 0 && sum_diff / opaque < 18 {
        for px in tile.chunks_exact_mut(4) {
            if px[3] > 0 {
                px[0] = ((px[0] as f32) * tint[0]).clamp(0.0, 255.0) as u8;
                px[1] = ((px[1] as f32) * tint[1]).clamp(0.0, 255.0) as u8;
                px[2] = ((px[2] as f32) * tint[2]).clamp(0.0, 255.0) as u8;
            }
        }
    }
}

fn map_block_texture_slots(stem: &str) -> Option<&'static [usize]> {
    Some(match stem {
        "stone" => &[0],
        "grass_block_top" | "grass_top" => &[1],
        "grass_block_side" | "grass_side" => &[2],
        "dirt" => &[3],
        "cobblestone" => &[4],
        "oak_log" | "log_oak" | "jungle_log" => &[5],
        "oak_log_top" | "log_oak_top" => &[6],
        "oak_leaves" | "leaves_oak" | "jungle_leaves" => &[7],
        "sand" => &[8],
        "water_still" => &[9],
        "snow" => &[10],
        "ice" => &[11],
        "oak_planks" | "planks_oak" => &[12],
        "glass" => &[13],
        "torch" | "torch_on" => &[14],
        "bedrock" => &[15],
        "coal_ore" => &[16],
        "iron_ore" => &[17],
        "gold_ore" => &[18],
        "diamond_ore" => &[19],
        "redstone_ore" => &[20],
        "emerald_ore" => &[21],
        "gravel" => &[22],
        "clay" => &[23],
        "cactus_side" => &[24],
        "short_grass" | "tall_grass" | "tallgrass" | "tall_grass_top" | "grass" => &[25],
        "poppy" | "flower_rose" | "rose" => &[26],
        "dandelion" | "flower_dandelion" => &[27],
        "bricks" | "brick" => &[28],
        "stone_bricks" | "stonebrick" => &[29],
        "white_wool" | "wool_colored_white" => &[30],
        "red_wool" | "wool_colored_red" => &[31],
        "blue_wool" | "wool_colored_blue" => &[32],
        "green_wool" | "wool_colored_green" => &[33],
        "yellow_wool" | "wool_colored_yellow" => &[34],
        "obsidian" => &[35],
        "magma" => &[36],
        "lava_still" => &[37],
        "farmland_moist" | "farmland_wet" | "farmland" | "farmland_dry" => &[38],
        "wheat_stage7" | "wheat_stage_7" => &[39],
        "chest_front" => &[40],
        "crafting_table_top" => &[41],
        "furnace_front" | "furnace_front_off" => &[42],
        "sandstone" | "sandstone_normal" => &[43],
        "pumpkin_side" => &[44],
        "hay_block_side" => &[45],
        "glowstone" => &[46],
        "spruce_log" | "log_spruce" | "dark_oak_log" => &[47],
        "cobweb" | "web" => &[48],
        "white_concrete" | "concrete_white" => &[49],
        "blue_concrete" | "concrete_blue" => &[50],
        "red_concrete" | "concrete_red" => &[51],
        "green_concrete" | "concrete_green" => &[52],
        "yellow_concrete" | "concrete_yellow" => &[53],
        "black_concrete" | "concrete_black" => &[54],
        "bookshelf" => &[55],
        "rail" | "rail_normal" => &[56],
        "anvil" | "anvil_base" => &[57],
        "enchanting_table_side" => &[58],
        "end_stone" => &[59],
        "netherrack" => &[60],
        "soul_sand" => &[61],
        "quartz_block_side" => &[62],
        "podzol_top" | "dirt_podzol_top" => &[63],
        "mycelium_top" => &[64],
        "redstone_dust_dot" | "redstone_dust_line0" => &[65],
        "redstone_torch" | "redstone_torch_on" => &[66],
        "lever" => &[67],
        "redstone_lamp" | "redstone_lamp_off" => &[68],
        "stone_pressure_plate" | "oak_pressure_plate" => &[69],
        "tnt_side" => &[70],
        "birch_log" | "log_birch" => &[71],
        "crafting_table_side" | "crafting_table_front" => &[72],
        "furnace_side" => &[73],
        "tnt_top" => &[74],
        "pumpkin_top" => &[75],
        "chest_side" => &[76],
        "spruce_leaves" | "leaves_spruce" | "dark_oak_leaves" => &[77],
        "birch_leaves" | "leaves_birch" => &[78],
        "red_sand" => &[79],
        "gold_block" => &[80],
        "diamond_block" => &[81],
        "emerald_block" => &[82],
        "iron_block" => &[83],
        "mossy_cobblestone" | "cobblestone_mossy" => &[84],
        "oak_door_top" | "door_wood_upper" => &[85],
        "oak_door_bottom" | "door_wood_lower" => &[86],
        "terracotta" | "hardened_clay" => &[87],
        "cactus_top" => &[88],
        "furnace_top" => &[89],
        "blackstone" => &[90],
        "ladder" => &[91],
        "lantern" => &[92],
        "sea_lantern" => &[93],
        "cake_side" => &[94],
        "carved_pumpkin" | "pumpkin_face_off" => &[95],
        "nether_portal" | "portal" => &[96],
        "end_portal_frame_top" | "endframe_top" => &[97],
        "end_portal_frame_side" | "endframe_side" => &[98],
        "end_portal_frame_eye" | "endframe_eye" => &[99],
        "crying_obsidian" => &[100],
        "crimson_nylium" => &[101],
        "warped_nylium" => &[102],
        "crimson_stem" => &[103],
        "nether_wart_block" => &[104],
        "warped_stem" => &[105],
        "warped_wart_block" => &[106],
        "basalt_side" => &[107],
        "shroomlight" => &[108],
        "nether_quartz_ore" | "quartz_ore" => &[109],
        "ancient_debris_side" => &[110],
        "chorus_plant" => &[111],
        "chorus_flower" => &[112],
        "purpur_block" => &[113],
        "nether_bricks" | "nether_brick" => &[114],
        "destroy_stage_0" => &[116],
        "destroy_stage_1" => &[117],
        "destroy_stage_2" => &[118],
        "destroy_stage_3" => &[119],
        "destroy_stage_4" => &[120],
        "destroy_stage_5" => &[121],
        "destroy_stage_6" => &[122],
        "destroy_stage_7" => &[123],
        "destroy_stage_8" => &[124],
        "destroy_stage_9" => &[125],
        "lapis_ore" => &[128],
        "lapis_block" => &[129],
        "brewing_stand" => &[130],
        "beacon" => &[131],
        "hopper_outside" => &[132],
        "hopper_top" => &[133],
        "piston_side" => &[134],
        "piston_top_normal" | "piston_top" => &[135],
        "piston_top_sticky" => &[136],
        "repeater" => &[137],
        "comparator" => &[139],
        "spawner" | "mob_spawner" => &[140],
        "oak_sapling" | "sapling_oak" => &[141],
        "enchanting_table_top" => &[142],
        "anvil_top" => &[143],
        _ => return None,
    })
}

fn map_item_texture_slot(stem: &str) -> Option<usize> {
    Some(match stem {
        "wooden_pickaxe" | "wood_pickaxe" => 0,
        "stone_pickaxe" => 1,
        "iron_pickaxe" => 2,
        "diamond_pickaxe" => 3,
        "netherite_pickaxe" => 4,
        "wooden_axe" | "wood_axe" => 5,
        "iron_axe" => 6,
        "diamond_axe" => 7,
        "wooden_shovel" | "wood_shovel" => 8,
        "iron_shovel" => 9,
        "wooden_sword" | "wood_sword" => 10,
        "iron_sword" => 11,
        "diamond_sword" => 12,
        "apple" => 13,
        "bread" => 14,
        "cooked_porkchop" | "porkchop_cooked" => 15,
        "porkchop" | "porkchop_raw" => 16,
        "beef" | "beef_raw" => 17,
        "cooked_beef" | "beef_cooked" => 18,
        "chicken" | "chicken_raw" => 19,
        "cooked_chicken" | "chicken_cooked" => 20,
        "cod" | "fish_cod_raw" => 21,
        "coal" => 22,
        "iron_ingot" => 23,
        "gold_ingot" => 24,
        "diamond" => 25,
        "emerald" => 26,
        "stick" => 27,
        "flint" => 28,
        "feather" => 29,
        "leather" => 30,
        "bone" => 31,
        "rotten_flesh" => 32,
        "string" => 33,
        "egg" => 35,
        "bucket" | "bucket_empty" => 36,
        "water_bucket" | "bucket_water" => 37,
        "lava_bucket" | "bucket_lava" => 38,
        "compass" | "compass_00" | "compass_16" => 39,
        "filled_map" | "map_filled" | "map" => 40,
        "clock" | "clock_00" => 41,
        "elytra" => 42,
        "firework_rocket" | "fireworks" => 43,
        "totem_of_undying" | "totem" => 45,
        "ender_pearl" => 46,
        "blaze_rod" => 47,
        "nether_star" => 48,
        "experience_bottle" => 49,
        "gunpowder" => 50,
        "golden_apple" | "apple_golden" => 51,
        "bow" | "bow_standby" => 52,
        "arrow" => 53,
        "flint_and_steel" => 54,
        "shears" => 55,
        "netherite_sword" => 56,
        "netherite_ingot" => 57,
        "carrot" => 58,
        "cookie" => 59,
        "slime_ball" | "slimeball" => 60,
        "amethyst_shard" => 61,
        "iron_helmet" => 62,
        "iron_chestplate" => 63,
        "diamond_helmet" => 64,
        "diamond_chestplate" => 65,
        "redstone" | "redstone_dust" => 78,
        "oak_door" | "door_wood" => 79,
        "oak_sign" | "sign" => 80,
        "red_bed" | "bed" => 81,
        "wheat" => 86,
        "ender_eye" => 90,
        "blaze_powder" => 91,
        "quartz" => 92,
        "gold_nugget" => 93,
        "lapis_lazuli" => 104,
        "bone_meal" => 105,
        "wooden_hoe" | "wood_hoe" => 106,
        "stone_hoe" => 107,
        "iron_hoe" => 108,
        "diamond_hoe" => 109,
        "oak_boat" | "boat" => 110,
        "minecart" | "minecart_normal" => 111,
        "glass_bottle" | "potion_bottle_empty" => 112,
        "potion" | "potion_bottle_drinkable" => 113,
        "iron_leggings" => 114,
        "iron_boots" => 115,
        "diamond_leggings" => 116,
        "diamond_boots" => 117,
        "netherite_helmet" => 118,
        "netherite_chestplate" => 119,
        "netherite_leggings" => 120,
        "netherite_boots" => 121,
        "nether_wart" => 122,
        "enchanted_book" | "book_enchanted" => 123,
        "brewing_stand" => 124,
        "hopper" => 125,
        "repeater" => 126,
        "comparator" => 127,
        _ => return None,
    })
}

// =========================================================================
// Datapack Recipe Parser (minecraft:crafting_shaped / shapeless / smelting)
// =========================================================================

fn parse_and_register_recipe(txt: &str) -> bool {
    let Some(j) = parse_json(txt) else { return false };
    let rtype = j.get("type").and_then(|v| v.as_str()).unwrap_or("");
    let short_type = rtype.strip_prefix("minecraft:").unwrap_or(rtype);

    let out_val = j.get("result");
    let (out_id, out_count) = match out_val {
        Some(JsonVal::Str(s)) => (mc_name_to_id(s), 1u32),
        Some(obj) => {
            let id_str = obj
                .get("id")
                .or_else(|| obj.get("item"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let cnt = obj
                .get("count")
                .and_then(|v| v.as_i64())
                .unwrap_or(1)
                .clamp(1, 64) as u32;
            (mc_name_to_id(id_str), cnt)
        }
        None => return false,
    };
    if out_id == 0 {
        return false;
    }

    match short_type {
        "crafting_shaped" => {
            let Some(pattern) = j.get("pattern").and_then(|v| v.as_array()) else {
                return false;
            };
            let Some(key_obj) = j.get("key").and_then(|v| v.as_object()) else {
                return false;
            };
            let mut key_map: HashMap<char, u16> = HashMap::new();
            for (k, v) in key_obj {
                if let Some(ch) = k.chars().next() {
                    let id = ingredient_to_id(v);
                    if id != 0 {
                        key_map.insert(ch, id);
                    }
                }
            }
            let mut rows = Vec::new();
            let mut max_w = 0usize;
            for line_v in pattern {
                let Some(line) = line_v.as_str() else { continue };
                let row: Vec<u16> = line
                    .chars()
                    .take(3)
                    .map(|c| if c == ' ' { 0 } else { *key_map.get(&c).unwrap_or(&0) })
                    .collect();
                max_w = max_w.max(row.len());
                rows.push(row);
            }
            if rows.is_empty() || max_w == 0 {
                return false;
            }
            let mut pat = [0u16; 9];
            for (ri, r) in rows.iter().take(3).enumerate() {
                for ci in 0..max_w.min(3) {
                    pat[ri * 3 + ci] = *r.get(ci).unwrap_or(&0);
                }
            }
            craft::add_custom_shaped(craft::Recipe {
                pattern: pat,
                out: out_id,
                count: out_count,
            });
            true
        }
        "crafting_shapeless" => {
            let Some(ings) = j.get("ingredients").and_then(|v| v.as_array()) else {
                return false;
            };
            let mut ids = Vec::new();
            for ing in ings {
                let id = ingredient_to_id(ing);
                if id != 0 {
                    ids.push(id);
                }
            }
            if ids.is_empty() || ids.len() > 9 {
                return false;
            }
            craft::add_custom_shapeless(craft::ShapelessRecipe {
                ingredients: ids,
                out: out_id,
                count: out_count,
            });
            true
        }
        "smelting" | "blasting" => {
            let Some(ing) = j.get("ingredient") else {
                return false;
            };
            let in_id = ingredient_to_id(ing);
            if in_id == 0 {
                return false;
            }
            craft::add_custom_smelt(in_id, out_id);
            true
        }
        _ => false,
    }
}

fn ingredient_to_id(v: &JsonVal) -> u16 {
    match v {
        JsonVal::Str(s) => mc_name_to_id(s),
        JsonVal::Arr(a) => a.first().map(ingredient_to_id).unwrap_or(0),
        JsonVal::Obj(_) => {
            if let Some(s) = v.get("item").and_then(|x| x.as_str()) {
                return mc_name_to_id(s);
            }
            if let Some(s) = v.get("tag").and_then(|x| x.as_str()) {
                return mc_name_to_id(s);
            }
            0
        }
        _ => 0,
    }
}

fn mc_name_to_id(raw: &str) -> u16 {
    let s = raw
        .trim()
        .trim_start_matches('#')
        .strip_prefix("minecraft:")
        .unwrap_or(raw.trim().trim_start_matches('#'));
    match s {
        "stone" | "smooth_stone" => world::STONE,
        "grass_block" => world::GRASS,
        "dirt" => world::DIRT,
        "cobblestone" | "stone_crafting_materials" | "stone_tool_materials" => world::COBBLE,
        "oak_log" | "logs" | "jungle_log" => world::LOG,
        "birch_log" => world::BIRCH_LOG,
        "spruce_log" | "dark_oak_log" => world::SPRUCE_LOG,
        "oak_planks" | "planks" | "spruce_planks" | "birch_planks" => world::PLANKS,
        "oak_leaves" | "leaves" => world::LEAVES,
        "sand" => world::SAND,
        "red_sand" => world::RED_SAND,
        "sandstone" => world::SANDSTONE,
        "gravel" => world::GRAVEL,
        "clay" | "clay_ball" => world::CLAY,
        "glass" => world::GLASS,
        "coal_ore" => world::COAL_ORE,
        "iron_ore" => world::IRON_ORE,
        "gold_ore" => world::GOLD_ORE,
        "diamond_ore" => world::DIAMOND_ORE,
        "redstone_ore" => world::REDSTONE_ORE,
        "emerald_ore" => world::EMERALD_ORE,
        "lapis_ore" => world::LAPIS_ORE,
        "iron_block" => world::IRON_BLOCK,
        "gold_block" => world::GOLD_BLOCK,
        "diamond_block" => world::DIAMOND_BLOCK,
        "emerald_block" => world::EMERALD_BLOCK,
        "lapis_block" => world::LAPIS_BLOCK,
        "obsidian" => world::OBSIDIAN,
        "crying_obsidian" => world::CRYING_OBSIDIAN,
        "bedrock" => world::BEDROCK,
        "glowstone" => world::GLOWSTONE,
        "netherrack" => world::NETHERRACK,
        "soul_sand" => world::SOUL_SAND,
        "nether_bricks" => world::NETHER_BRICKS,
        "end_stone" => world::END_STONE,
        "bricks" => world::BRICK,
        "stone_bricks" => world::STONE_BRICKS,
        "bookshelf" => world::BOOKSHELF,
        "tnt" => world::TNT,
        "torch" => world::TORCH,
        "redstone_torch" => world::REDSTONE_TORCH,
        "redstone" => world::REDSTONE_DUST,
        "crafting_table" => world::CRAFTING,
        "furnace" => world::FURNACE,
        "chest" => world::CHEST,
        "anvil" => world::ANVIL,
        "enchanting_table" => world::ENCHANTING,
        "brewing_stand" => world::BREWING_STAND,
        "hopper" => world::HOPPER,
        "piston" => world::PISTON,
        "sticky_piston" => world::STICKY_PISTON,
        "repeater" => world::REPEATER,
        "comparator" => world::COMPARATOR,
        "beacon" => world::BEACON,
        "rail" => world::RAIL,
        "white_wool" | "wool" => world::WOOL_WHITE,
        "red_wool" => world::WOOL_RED,
        "blue_wool" => world::WOOL_BLUE,
        "green_wool" => world::WOOL_GREEN,
        "yellow_wool" => world::WOOL_YELLOW,
        "wooden_pickaxe" => items::WOOD_PICK,
        "stone_pickaxe" => items::STONE_PICK,
        "iron_pickaxe" => items::IRON_PICK,
        "diamond_pickaxe" => items::DIAMOND_PICK,
        "netherite_pickaxe" => items::NETHERITE_PICK,
        "wooden_axe" => items::WOOD_AXE,
        "iron_axe" => items::IRON_AXE,
        "diamond_axe" => items::DIAMOND_AXE,
        "wooden_shovel" => items::WOOD_SHOVEL,
        "iron_shovel" => items::IRON_SHOVEL,
        "wooden_sword" => items::WOOD_SWORD,
        "iron_sword" => items::IRON_SWORD,
        "diamond_sword" => items::DIAMOND_SWORD,
        "netherite_sword" => items::NETHERITE_SWORD,
        "wooden_hoe" => items::WOOD_HOE,
        "stone_hoe" => items::STONE_HOE,
        "iron_hoe" => items::IRON_HOE,
        "diamond_hoe" => items::DIAMOND_HOE,
        "apple" => items::APPLE,
        "golden_apple" => items::GOLDEN_APPLE,
        "bread" => items::BREAD,
        "wheat" => world::WHEAT,
        "carrot" => items::CARROT,
        "cookie" => items::COOKIE,
        "cooked_porkchop" => items::COOKED_PORK,
        "porkchop" => items::RAW_PORK,
        "beef" => items::RAW_BEEF,
        "cooked_beef" => items::STEAK,
        "chicken" => items::RAW_CHICKEN,
        "cooked_chicken" => items::COOKED_CHICKEN,
        "cod" => items::FISH,
        "coal" | "charcoal" => items::COAL,
        "iron_ingot" => items::IRON_INGOT,
        "gold_ingot" => items::GOLD_INGOT,
        "gold_nugget" => items::GOLD_NUGGET,
        "diamond" => items::DIAMOND,
        "emerald" => items::EMERALD,
        "lapis_lazuli" => items::LAPIS_LAZULI,
        "quartz" => items::NETHER_QUARTZ,
        "netherite_ingot" => items::NETHERITE_INGOT,
        "stick" => items::STICK,
        "flint" => items::FLINT,
        "feather" => items::FEATHER,
        "leather" => items::LEATHER,
        "bone" => items::BONE,
        "bone_meal" => items::BONE_MEAL,
        "rotten_flesh" => items::ROTTEN_FLESH,
        "string" => items::STRING,
        "egg" => items::EGG,
        "bucket" => items::BUCKET,
        "water_bucket" => items::WATER_BUCKET,
        "lava_bucket" => items::LAVA_BUCKET,
        "compass" => items::COMPASS,
        "map" | "filled_map" => items::MAP,
        "clock" => items::CLOCK,
        "elytra" => items::ELYTRA,
        "firework_rocket" => items::FIREWORK,
        "shield" => items::SHIELD,
        "totem_of_undying" => items::TOTEM,
        "ender_pearl" => items::ENDER_PEARL,
        "ender_eye" => items::EYE_OF_ENDER,
        "blaze_rod" => items::BLAZE_ROD,
        "blaze_powder" => items::BLAZE_POWDER,
        "nether_star" => items::NETHER_STAR,
        "experience_bottle" => items::XP_BOTTLE,
        "gunpowder" => items::GUNPOWDER,
        "bow" => items::BOW,
        "arrow" => items::ARROW,
        "flint_and_steel" => items::FLINT_AND_STEEL,
        "shears" => items::SHEARS,
        "slime_ball" => items::SLIMEBALL,
        "amethyst_shard" => items::AMETHYST,
        "iron_helmet" => items::IRON_HELMET,
        "iron_chestplate" => items::IRON_CHESTPLATE,
        "iron_leggings" => items::IRON_LEGGINGS,
        "iron_boots" => items::IRON_BOOTS,
        "diamond_helmet" => items::DIAMOND_HELMET,
        "diamond_chestplate" => items::DIAMOND_CHESTPLATE,
        "diamond_leggings" => items::DIAMOND_LEGGINGS,
        "diamond_boots" => items::DIAMOND_BOOTS,
        "netherite_helmet" => items::NETHERITE_HELMET,
        "netherite_chestplate" => items::NETHERITE_CHESTPLATE,
        "netherite_leggings" => items::NETHERITE_LEGGINGS,
        "netherite_boots" => items::NETHERITE_BOOTS,
        "oak_boat" => items::BOAT,
        "minecart" => items::MINECART,
        "glass_bottle" => items::GLASS_BOTTLE,
        "enchanted_book" => items::ENCHANTED_BOOK,
        _ => 0,
    }
}

// =========================================================================
// Pure-Rust ZIP Reader + RFC 1951 DEFLATE Inflate
// =========================================================================

#[inline]
fn le16(b: &[u8], i: usize) -> u16 {
    u16::from_le_bytes([b[i], b[i + 1]])
}

#[inline]
fn le32(b: &[u8], i: usize) -> u32 {
    u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]])
}

pub fn read_zip_entries(data: &[u8]) -> Result<Vec<(String, Vec<u8>)>, String> {
    if data.len() < 22 {
        return Err(String::from("File too small to be a ZIP archive"));
    }
    // Locate End of Central Directory (PK\x05\x06) scanning backwards up to 64 KiB
    let min_pos = data.len().saturating_sub(65560);
    let mut eocd = None;
    let mut pos = data.len() - 22;
    loop {
        if &data[pos..pos + 4] == b"PK\x05\x06" {
            let clen = le16(data, pos + 20) as usize;
            if pos + 22 + clen <= data.len() {
                eocd = Some(pos);
                break;
            }
        }
        if pos <= min_pos {
            break;
        }
        pos -= 1;
    }

    let mut out = Vec::new();
    if let Some(ep) = eocd {
        let total_entries = le16(data, ep + 10) as usize;
        let cd_offset = le32(data, ep + 16) as usize;
        let mut p = cd_offset;
        for _ in 0..total_entries {
            if p + 46 > data.len() || &data[p..p + 4] != b"PK\x01\x02" {
                break;
            }
            let method = le16(data, p + 10);
            let comp_sz = le32(data, p + 20) as usize;
            let uncomp_sz = le32(data, p + 24) as usize;
            let name_len = le16(data, p + 28) as usize;
            let extra_len = le16(data, p + 30) as usize;
            let cmt_len = le16(data, p + 32) as usize;
            let lfh_off = le32(data, p + 42) as usize;
            if p + 46 + name_len > data.len() {
                break;
            }
            let name = String::from_utf8_lossy(&data[p + 46..p + 46 + name_len]).into_owned();
            p += 46 + name_len + extra_len + cmt_len;
            if name.ends_with('/') {
                continue;
            }
            if lfh_off + 30 <= data.len() && &data[lfh_off..lfh_off + 4] == b"PK\x03\x04" {
                let ln = le16(data, lfh_off + 26) as usize;
                let le = le16(data, lfh_off + 28) as usize;
                let dstart = lfh_off + 30 + ln + le;
                let dend = dstart.saturating_add(comp_sz).min(data.len());
                if dstart <= dend {
                    let payload = &data[dstart..dend];
                    let content = match method {
                        0 => Ok(payload.to_vec()),
                        8 => inflate_raw(payload, uncomp_sz),
                        _ => Err(String::from("Unsupported ZIP compression")),
                    };
                    if let Ok(bytes) = content {
                        out.push((name, bytes));
                    }
                }
            }
        }
        return Ok(out);
    }

    // Fallback: sequential local file header scan
    let mut p = 0usize;
    while p + 30 <= data.len() {
        if &data[p..p + 4] != b"PK\x03\x04" {
            break;
        }
        let method = le16(data, p + 8);
        let comp_sz = le32(data, p + 18) as usize;
        let uncomp_sz = le32(data, p + 22) as usize;
        let name_len = le16(data, p + 26) as usize;
        let extra_len = le16(data, p + 28) as usize;
        let dstart = p + 30 + name_len + extra_len;
        let dend = dstart.saturating_add(comp_sz);
        if dend > data.len() || comp_sz == 0 && (le16(data, p + 6) & 0x08) != 0 {
            break;
        }
        let name = String::from_utf8_lossy(&data[p + 30..p + 30 + name_len]).into_owned();
        if !name.ends_with('/') {
            let payload = &data[dstart..dend];
            if let Ok(bytes) = match method {
                0 => Ok(payload.to_vec()),
                8 => inflate_raw(payload, uncomp_sz),
                _ => Err(String::from("Unsupported method")),
            } {
                out.push((name, bytes));
            }
        }
        p = dend;
    }
    Ok(out)
}

struct BitReader<'a> {
    data: &'a [u8],
    byte_pos: usize,
    bit_buf: u32,
    bit_cnt: u8,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            byte_pos: 0,
            bit_buf: 0,
            bit_cnt: 0,
        }
    }

    #[inline]
    fn ensure(&mut self, n: u8) -> Result<(), String> {
        while self.bit_cnt < n {
            if self.byte_pos >= self.data.len() {
                return Err(String::from("Unexpected EOF in deflate stream"));
            }
            self.bit_buf |= (self.data[self.byte_pos] as u32) << self.bit_cnt;
            self.byte_pos += 1;
            self.bit_cnt += 8;
        }
        Ok(())
    }

    #[inline]
    fn read_bits(&mut self, n: u8) -> Result<u32, String> {
        if n == 0 {
            return Ok(0);
        }
        self.ensure(n)?;
        let val = self.bit_buf & ((1u32 << n) - 1);
        self.bit_buf >>= n;
        self.bit_cnt -= n;
        Ok(val)
    }

    fn align_byte(&mut self) {
        let drop = self.bit_cnt & 7;
        self.bit_buf >>= drop;
        self.bit_cnt -= drop;
    }

    fn read_u16_aligned(&mut self) -> Result<u16, String> {
        self.align_byte();
        let lo = self.read_bits(8)? as u16;
        let hi = self.read_bits(8)? as u16;
        Ok(lo | (hi << 8))
    }
}

struct HuffTree {
    counts: [u16; 16],
    symbols: Vec<u16>,
}

impl HuffTree {
    fn build(lengths: &[u8]) -> Result<Self, String> {
        let mut counts = [0u16; 16];
        for &l in lengths {
            if (l as usize) < 16 {
                counts[l as usize] += 1;
            }
        }
        counts[0] = 0;
        let mut offs = [0u16; 16];
        let mut sum = 0u16;
        for bits in 1..=15 {
            offs[bits] = sum;
            sum = sum.saturating_add(counts[bits]);
        }
        let mut symbols = vec![0u16; sum as usize];
        for (sym, &l) in lengths.iter().enumerate() {
            if l != 0 && (l as usize) < 16 {
                let idx = offs[l as usize] as usize;
                offs[l as usize] += 1;
                if idx < symbols.len() {
                    symbols[idx] = sym as u16;
                }
            }
        }
        Ok(Self { counts, symbols })
    }

    #[inline]
    fn decode(&self, br: &mut BitReader<'_>) -> Result<u16, String> {
        let mut code = 0i32;
        let mut first = 0i32;
        let mut index = 0i32;
        for len in 1..=15 {
            code |= br.read_bits(1)? as i32;
            let count = self.counts[len] as i32;
            if code - count < first {
                let si = (index + (code - first)) as usize;
                return self
                    .symbols
                    .get(si)
                    .copied()
                    .ok_or_else(|| String::from("Huffman symbol OOB"));
            }
            index += count;
            first += count;
            first <<= 1;
            code <<= 1;
        }
        Err(String::from("Invalid Huffman code"))
    }
}

pub fn inflate_raw(data: &[u8], hint_len: usize) -> Result<Vec<u8>, String> {
    let mut br = BitReader::new(data);
    let mut out = Vec::with_capacity(hint_len.min(4 * 1024 * 1024));

    let len_base: [usize; 29] = [
        3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115,
        131, 163, 195, 227, 258,
    ];
    let len_ext: [u8; 29] = [
        0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
    ];
    let dist_base: [usize; 30] = [
        1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025,
        1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
    ];
    let dist_ext: [u8; 30] = [
        0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12,
        13, 13,
    ];

    loop {
        let bfinal = br.read_bits(1)?;
        let btype = br.read_bits(2)?;
        if btype == 0 {
            let len = br.read_u16_aligned()? as usize;
            let _nlen = br.read_u16_aligned()?;
            for _ in 0..len {
                out.push(br.read_bits(8)? as u8);
            }
        } else if btype == 1 || btype == 2 {
            let (lit_tree, dist_tree) = if btype == 1 {
                let mut ll = [0u8; 288];
                for x in 0..=143 { ll[x] = 8; }
                for x in 144..=255 { ll[x] = 9; }
                for x in 256..=279 { ll[x] = 7; }
                for x in 280..=287 { ll[x] = 8; }
                let dl = [5u8; 32];
                (HuffTree::build(&ll)?, HuffTree::build(&dl)?)
            } else {
                let hlit = br.read_bits(5)? as usize + 257;
                let hdist = br.read_bits(5)? as usize + 1;
                let hclen = br.read_bits(4)? as usize + 4;
                let order = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];
                let mut cl_lens = [0u8; 19];
                for i in 0..hclen {
                    cl_lens[order[i]] = br.read_bits(3)? as u8;
                }
                let cl_tree = HuffTree::build(&cl_lens)?;
                let total = hlit + hdist;
                let mut lens = Vec::with_capacity(total);
                while lens.len() < total {
                    let sym = cl_tree.decode(&mut br)?;
                    match sym {
                        0..=15 => lens.push(sym as u8),
                        16 => {
                            let prev = *lens.last().unwrap_or(&0);
                            let rep = br.read_bits(2)? as usize + 3;
                            for _ in 0..rep {
                                if lens.len() < total { lens.push(prev); }
                            }
                        }
                        17 => {
                            let rep = br.read_bits(3)? as usize + 3;
                            for _ in 0..rep {
                                if lens.len() < total { lens.push(0); }
                            }
                        }
                        18 => {
                            let rep = br.read_bits(7)? as usize + 11;
                            for _ in 0..rep {
                                if lens.len() < total { lens.push(0); }
                            }
                        }
                        _ => return Err(String::from("Bad code length sym")),
                    }
                }
                (HuffTree::build(&lens[..hlit])?, HuffTree::build(&lens[hlit..])?)
            };

            loop {
                let sym = lit_tree.decode(&mut br)? as usize;
                if sym < 256 {
                    out.push(sym as u8);
                } else if sym == 256 {
                    break;
                } else {
                    let li = sym - 257;
                    if li >= len_base.len() {
                        return Err(String::from("Invalid deflate length code"));
                    }
                    let length = len_base[li] + br.read_bits(len_ext[li])? as usize;
                    let di = dist_tree.decode(&mut br)? as usize;
                    if di >= dist_base.len() {
                        return Err(String::from("Invalid deflate dist code"));
                    }
                    let dist = dist_base[di] + br.read_bits(dist_ext[di])? as usize;
                    if dist == 0 || dist > out.len() {
                        return Err(String::from("Deflate back-ref out of range"));
                    }
                    for _ in 0..length {
                        let b = out[out.len() - dist];
                        out.push(b);
                    }
                }
            }
        } else {
            return Err(String::from("Reserved deflate block type"));
        }
        if bfinal != 0 {
            break;
        }
    }
    Ok(out)
}

pub fn inflate_zlib(data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < 6 {
        return Err(String::from("Zlib stream too short"));
    }
    let cmf = data[0];
    let flg = data[1];
    if (cmf & 0x0f) != 8 {
        return Err(String::from("Unsupported zlib compression method"));
    }
    let fdict = (flg & 0x20) != 0;
    let start = if fdict { 6 } else { 2 };
    inflate_raw(&data[start..], 0)
}

// =========================================================================
// Pure-Rust PNG Decoder + Downsampler -> 16x16 RGBA
// =========================================================================

pub fn decode_png(data: &[u8]) -> Result<(usize, usize, Vec<u8>), String> {
    if data.len() < 33 || &data[0..8] != b"\x89PNG\r\n\x1a\n" {
        return Err(String::from("Invalid PNG signature"));
    }
    let mut pos = 8usize;
    let mut w = 0usize;
    let mut h = 0usize;
    let mut bit_depth = 0u8;
    let mut color_type = 0u8;
    let mut plte: Vec<[u8; 3]> = Vec::new();
    let mut trns: Vec<u8> = Vec::new();
    let mut idat = Vec::new();

    while pos + 12 <= data.len() {
        let len = u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        let ctype = &data[pos + 4..pos + 8];
        let dstart = pos + 8;
        let dend = dstart.saturating_add(len);
        if dend + 4 > data.len() {
            break;
        }
        let chunk = &data[dstart..dend];
        match ctype {
            b"IHDR" if len >= 13 => {
                w = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) as usize;
                h = u32::from_be_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]) as usize;
                bit_depth = chunk[8];
                color_type = chunk[9];
                if chunk[12] != 0 {
                    return Err(String::from("Interlaced PNGs not supported"));
                }
            }
            b"PLTE" => {
                for rgb in chunk.chunks_exact(3) {
                    plte.push([rgb[0], rgb[1], rgb[2]]);
                }
            }
            b"tRNS" => {
                trns = chunk.to_vec();
            }
            b"IDAT" => idat.extend_from_slice(chunk),
            b"IEND" => break,
            _ => {}
        }
        pos = dend + 4;
    }

    if w == 0 || h == 0 || w > 4096 || h > 16384 {
        return Err(String::from("Invalid PNG dimensions"));
    }
    let raw = inflate_zlib(&idat)?;

    let channels = match color_type {
        0 => 1usize,
        2 => 3usize,
        3 => 1usize,
        4 => 2usize,
        6 => 4usize,
        _ => return Err(String::from("Unsupported PNG color type")),
    };
    let bpp = ((channels * bit_depth as usize) + 7) / 8;
    let stride = (w * channels * bit_depth as usize + 7) / 8;
    if raw.len() < h * (stride + 1) {
        return Err(String::from("Truncated PNG image data"));
    }

    let mut recon = vec![0u8; h * stride];
    for y in 0..h {
        let src = &raw[y * (stride + 1)..(y + 1) * (stride + 1)];
        let filter = src[0];
        let row = &src[1..];
        let row_off = y * stride;
        for x in 0..stride {
            let a = if x >= bpp { recon[row_off + x - bpp] } else { 0 };
            let b = if y > 0 { recon[(y - 1) * stride + x] } else { 0 };
            let c = if y > 0 && x >= bpp {
                recon[(y - 1) * stride + x - bpp]
            } else {
                0
            };
            let v = match filter {
                0 => row[x],
                1 => row[x].wrapping_add(a),
                2 => row[x].wrapping_add(b),
                3 => row[x].wrapping_add(((a as u16 + b as u16) >> 1) as u8),
                4 => row[x].wrapping_add(paeth(a, b, c)),
                _ => return Err(String::from("Invalid PNG filter")),
            };
            recon[row_off + x] = v;
        }
    }

    let mut rgba = vec![0u8; w * h * 4];
    for y in 0..h {
        let row = &recon[y * stride..(y + 1) * stride];
        for x in 0..w {
            let di = (y * w + x) * 4;
            match (color_type, bit_depth) {
                (6, 8) => {
                    let si = x * 4;
                    rgba[di..di + 4].copy_from_slice(&row[si..si + 4]);
                }
                (6, 16) => {
                    let si = x * 8;
                    rgba[di] = row[si];
                    rgba[di + 1] = row[si + 2];
                    rgba[di + 2] = row[si + 4];
                    rgba[di + 3] = row[si + 6];
                }
                (2, 8) => {
                    let si = x * 3;
                    rgba[di] = row[si];
                    rgba[di + 1] = row[si + 1];
                    rgba[di + 2] = row[si + 2];
                    rgba[di + 3] = 255;
                }
                (2, 16) => {
                    let si = x * 6;
                    rgba[di] = row[si];
                    rgba[di + 1] = row[si + 2];
                    rgba[di + 2] = row[si + 4];
                    rgba[di + 3] = 255;
                }
                (4, 8) => {
                    let si = x * 2;
                    rgba[di] = row[si];
                    rgba[di + 1] = row[si];
                    rgba[di + 2] = row[si];
                    rgba[di + 3] = row[si + 1];
                }
                (0, 8) => {
                    let g = row[x];
                    rgba[di] = g;
                    rgba[di + 1] = g;
                    rgba[di + 2] = g;
                    rgba[di + 3] = 255;
                }
                (0, bd @ (1 | 2 | 4)) => {
                    let g = extract_subbyte(row, x, bd);
                    let maxv = (1u32 << bd) - 1;
                    let v = ((g as u32 * 255) / maxv) as u8;
                    rgba[di] = v;
                    rgba[di + 1] = v;
                    rgba[di + 2] = v;
                    rgba[di + 3] = 255;
                }
                (3, bd @ (1 | 2 | 4 | 8)) => {
                    let idx = extract_subbyte(row, x, bd) as usize;
                    let rgb = plte.get(idx).copied().unwrap_or([255, 0, 255]);
                    let a = trns.get(idx).copied().unwrap_or(255);
                    rgba[di] = rgb[0];
                    rgba[di + 1] = rgb[1];
                    rgba[di + 2] = rgb[2];
                    rgba[di + 3] = a;
                }
                _ => return Err(String::from("Unsupported PNG bit depth")),
            }
        }
    }

    Ok((w, h, rgba))
}

#[inline]
fn paeth(a: u8, b: u8, c: u8) -> u8 {
    let p = a as i32 + b as i32 - c as i32;
    let pa = (p - a as i32).abs();
    let pb = (p - b as i32).abs();
    let pc = (p - c as i32).abs();
    if pa <= pb && pa <= pc { a } else if pb <= pc { b } else { c }
}

#[inline]
fn extract_subbyte(row: &[u8], x: usize, bd: u8) -> u8 {
    if bd == 8 {
        return row[x];
    }
    let ppb = 8 / bd as usize;
    let b = row[x / ppb];
    let shift = 8 - bd as usize * ((x % ppb) + 1);
    (b >> shift) & ((1u8 << bd) - 1)
}

/// Crop animated strip textures (e.g. 16x512 water_still) to frame 0 (w x w),
/// then box-filter (for HD packs 32x/64x/128x) or nearest-sample into 16x16 RGBA.
pub fn downsample_to_16x16(w: usize, h: usize, rgba: &[u8]) -> [u8; 1024] {
    let mut out = [0u8; 1024];
    let src_h = if h > w && h % w == 0 { w } else { h };
    if w == 16 && src_h == 16 {
        out.copy_from_slice(&rgba[..1024]);
        return out;
    }
    if w >= 16 && src_h >= 16 {
        for ty in 0..16 {
            let y0 = (ty * src_h) / 16;
            let y1 = (((ty + 1) * src_h) / 16).max(y0 + 1);
            for tx in 0..16 {
                let x0 = (tx * w) / 16;
                let x1 = (((tx + 1) * w) / 16).max(x0 + 1);
                let mut sr = 0u32;
                let mut sg = 0u32;
                let mut sb = 0u32;
                let mut sa = 0u32;
                let mut cnt = 0u32;
                let mut w_rgb = 0u32;
                for sy in y0..y1 {
                    for sx in x0..x1 {
                        let i = (sy * w + sx) * 4;
                        let a = rgba[i + 3] as u32;
                        if a > 0 {
                            sr += rgba[i] as u32 * a;
                            sg += rgba[i + 1] as u32 * a;
                            sb += rgba[i + 2] as u32 * a;
                            w_rgb += a;
                        }
                        sa += a;
                        cnt += 1;
                    }
                }
                let di = (ty * 16 + tx) * 4;
                if w_rgb > 0 {
                    out[di] = (sr / w_rgb).min(255) as u8;
                    out[di + 1] = (sg / w_rgb).min(255) as u8;
                    out[di + 2] = (sb / w_rgb).min(255) as u8;
                }
                out[di + 3] = (sa / cnt.max(1)).min(255) as u8;
            }
        }
    } else {
        for ty in 0..16 {
            let sy = (ty * src_h) / 16;
            for tx in 0..16 {
                let sx = (tx * w) / 16;
                let si = (sy * w + sx) * 4;
                let di = (ty * 16 + tx) * 4;
                out[di..di + 4].copy_from_slice(&rgba[si..si + 4]);
            }
        }
    }
    out
}

// =========================================================================
// Minimal Recursive JSON Parser
// =========================================================================

#[allow(dead_code)]
#[derive(Clone, Debug)]
enum JsonVal {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<JsonVal>),
    Obj(Vec<(String, JsonVal)>),
}

impl JsonVal {
    fn get(&self, key: &str) -> Option<&JsonVal> {
        if let JsonVal::Obj(entries) = self {
            entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
        } else {
            None
        }
    }

    fn as_str(&self) -> Option<&str> {
        if let JsonVal::Str(s) = self { Some(s.as_str()) } else { None }
    }

    fn as_i64(&self) -> Option<i64> {
        if let JsonVal::Num(n) = self { Some(*n as i64) } else { None }
    }

    fn as_array(&self) -> Option<&[JsonVal]> {
        if let JsonVal::Arr(a) = self { Some(a.as_slice()) } else { None }
    }

    fn as_object(&self) -> Option<&[(String, JsonVal)]> {
        if let JsonVal::Obj(o) = self { Some(o.as_slice()) } else { None }
    }
}

fn json_text_to_string(v: &JsonVal) -> String {
    match v {
        JsonVal::Str(s) => s.clone(),
        JsonVal::Obj(_) => v
            .get("text")
            .and_then(|x| x.as_str())
            .unwrap_or("Resource Pack")
            .to_string(),
        JsonVal::Arr(a) => a.iter().map(json_text_to_string).collect::<Vec<_>>().join(""),
        _ => String::new(),
    }
}

fn parse_json(src: &str) -> Option<JsonVal> {
    let bytes = src.as_bytes();
    let mut i = 0usize;
    let val = parse_json_val(bytes, &mut i)?;
    Some(val)
}

fn skip_ws(b: &[u8], i: &mut usize) {
    while *i < b.len() && matches!(b[*i], b' ' | b'\t' | b'\r' | b'\n') {
        *i += 1;
    }
}

fn parse_json_val(b: &[u8], i: &mut usize) -> Option<JsonVal> {
    skip_ws(b, i);
    if *i >= b.len() {
        return None;
    }
    match b[*i] {
        b'"' => parse_json_str(b, i).map(JsonVal::Str),
        b'{' => {
            *i += 1;
            let mut obj = Vec::new();
            skip_ws(b, i);
            if *i < b.len() && b[*i] == b'}' {
                *i += 1;
                return Some(JsonVal::Obj(obj));
            }
            loop {
                skip_ws(b, i);
                let key = parse_json_str(b, i)?;
                skip_ws(b, i);
                if *i >= b.len() || b[*i] != b':' {
                    return None;
                }
                *i += 1;
                let val = parse_json_val(b, i)?;
                obj.push((key, val));
                skip_ws(b, i);
                if *i < b.len() && b[*i] == b',' {
                    *i += 1;
                    continue;
                }
                if *i < b.len() && b[*i] == b'}' {
                    *i += 1;
                    break;
                }
                return None;
            }
            Some(JsonVal::Obj(obj))
        }
        b'[' => {
            *i += 1;
            let mut arr = Vec::new();
            skip_ws(b, i);
            if *i < b.len() && b[*i] == b']' {
                *i += 1;
                return Some(JsonVal::Arr(arr));
            }
            loop {
                let val = parse_json_val(b, i)?;
                arr.push(val);
                skip_ws(b, i);
                if *i < b.len() && b[*i] == b',' {
                    *i += 1;
                    continue;
                }
                if *i < b.len() && b[*i] == b']' {
                    *i += 1;
                    break;
                }
                return None;
            }
            Some(JsonVal::Arr(arr))
        }
        b't' => {
            if b[*i..].starts_with(b"true") {
                *i += 4;
                Some(JsonVal::Bool(true))
            } else {
                None
            }
        }
        b'f' => {
            if b[*i..].starts_with(b"false") {
                *i += 5;
                Some(JsonVal::Bool(false))
            } else {
                None
            }
        }
        b'n' => {
            if b[*i..].starts_with(b"null") {
                *i += 4;
                Some(JsonVal::Null)
            } else {
                None
            }
        }
        b'-' | b'0'..=b'9' => {
            let start = *i;
            *i += 1;
            while *i < b.len()
                && matches!(b[*i], b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-')
            {
                *i += 1;
            }
            let s = core::str::from_utf8(&b[start..*i]).ok()?;
            let n = s.parse::<f64>().ok()?;
            Some(JsonVal::Num(n))
        }
        _ => None,
    }
}

fn parse_json_str(b: &[u8], i: &mut usize) -> Option<String> {
    if *i >= b.len() || b[*i] != b'"' {
        return None;
    }
    *i += 1;
    let mut s = String::new();
    while *i < b.len() {
        let c = b[*i];
        *i += 1;
        if c == b'"' {
            return Some(s);
        }
        if c == b'\\' {
            if *i >= b.len() {
                return None;
            }
            let esc = b[*i];
            *i += 1;
            match esc {
                b'"' => s.push('"'),
                b'\\' => s.push('\\'),
                b'/' => s.push('/'),
                b'n' => s.push('\n'),
                b'r' => s.push('\r'),
                b't' => s.push('\t'),
                b'b' => s.push('\x08'),
                b'f' => s.push('\x0c'),
                b'u' => {
                    if *i + 4 <= b.len() {
                        if let Ok(hx) = core::str::from_utf8(&b[*i..*i + 4]) {
                            if let Ok(cp) = u32::from_str_radix(hx, 16) {
                                if let Some(ch) = char::from_u32(cp) {
                                    s.push(ch);
                                }
                            }
                        }
                        *i += 4;
                    }
                }
                other => s.push(other as char),
            }
        } else {
            let start = *i - 1;
            while *i < b.len() && b[*i] >= 0x80 && (b[*i] & 0xC0) == 0x80 {
                *i += 1;
            }
            if let Ok(part) = core::str::from_utf8(&b[start..*i]) {
                s.push_str(part);
            }
        }
    }
    None
}

fn map_minecraft_sound_sfx(ln: &str) -> Option<Sfx> {
    if ln.contains("/click") || ln.contains("ui/button/click") { return Some(Sfx::Click); }
    if ln.contains("random/pop") { return Some(Sfx::Pickup); }
    if ln.contains("random/levelup") || ln.contains("orb") { return Some(Sfx::LevelUp); }
    if ln.contains("random/explode") { return Some(Sfx::Explode); }
    if ln.contains("door_open") || ln.contains("door/open") { return Some(Sfx::Door); }
    if ln.contains("liquid/splash") || ln.contains("swim") { return Some(Sfx::Splash); }
    if ln.contains("dig/stone") || ln.contains("block/stone/break") { return Some(Sfx::Break); }
    if ln.contains("dig/grass") || ln.contains("dig/wood") { return Some(Sfx::Place); }
    if ln.contains("step/grass") { return Some(Sfx::StepGrass); }
    if ln.contains("step/stone") { return Some(Sfx::StepStone); }
    if ln.contains("step/wood") { return Some(Sfx::StepWood); }
    if ln.contains("step/sand") || ln.contains("step/gravel") { return Some(Sfx::StepSand); }
    if ln.contains("damage/hit") || ln.contains("player/hurt") { return Some(Sfx::Hurt); }
    if ln.contains("damage/fallbig") || ln.contains("damage/fallsmall") { return Some(Sfx::Land); }
    if ln.contains("mob/cow/say") { return Some(Sfx::Cow); }
    if ln.contains("mob/sheep/say") { return Some(Sfx::Sheep); }
    if ln.contains("mob/pig/say") { return Some(Sfx::Pig); }
    if ln.contains("mob/chicken/say") { return Some(Sfx::Chicken); }
    if ln.contains("mob/zombie/say") { return Some(Sfx::Zombie); }
    if ln.contains("mob/skeleton/say") { return Some(Sfx::Skeleton); }
    if ln.contains("mob/spider/say") { return Some(Sfx::Spider); }
    if ln.contains("mob/creeper/say") || ln.contains("random/fuse") { return Some(Sfx::Creeper); }
    if ln.contains("mob/endermen/idle") || ln.contains("mob/enderman/idle") { return Some(Sfx::Enderman); }
    if ln.contains("mob/") && ln.contains("hurt") { return Some(Sfx::MobHurt); }
    if ln.contains("mob/") && ln.contains("death") { return Some(Sfx::MobDie); }
    None
}

