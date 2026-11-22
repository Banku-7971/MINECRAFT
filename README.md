# Minecraft (Rust Edition)

A Minecraft-inspired voxel sandbox written from scratch in **Rust** with zero
external crate dependencies. It builds both as a native Windows game and as a
WebAssembly game that runs entirely in the browser.

## Play in the browser

**Live build: <https://rustcraft-vert.vercel.app>**

The browser build is a static site: Rust compiles to a single
`web/rustcraft.wasm`, and `web/game.js` drives it with `requestAnimationFrame`.
There is no game server, so it can be hosted for free on Vercel (or any static
host) and every push to the connected Git branch deploys automatically.

```bash
npm run build:wasm   # requires Rust + wasm32-unknown-unknown
npm run start:web    # http://localhost:3000
```

The compiled `web/rustcraft.wasm` is committed, so **Vercel does not need a Rust
toolchain**. Its build step only verifies the artifact and serves `web/` as the
output directory (see `vercel.json`).

Browser limitations:

- Singleplayer only. Browsers cannot open the raw TCP sockets used by the
  desktop LAN mode.
- Audio is currently desktop-only.
- The software renderer defaults to a 160x90 scene upscaled to 480x270. Change
  it in **Options → Pixel Scale** if you want more detail or more speed.

## Native Windows game

Download the pre-built Windows binaries from **[GitHub Releases](https://github.com/Banku-7971/MINECRAFT/releases/latest)** (built automatically by `.github/workflows/release-windows.yml`), or compile them locally into `runtime/`:

- **`Minecraft.exe`** (~3 MB) - standalone portable Windows 64-bit GUI
  application, zero external DLL or Java dependencies.
- **`Minecraft-Setup.exe`** (~5 MB) - native Windows GUI installer. It installs
  `Minecraft.exe` to `%LOCALAPPDATA%\Minecraft\`, creates Desktop and Start Menu
  shortcuts, registers **Minecraft** in Windows Apps & Features (with uninstall
  support), and offers to launch immediately.

> **Why is it ~3–5 MB instead of 200+ MB?** Official Minecraft Java Edition
> ships the Java Virtual Machine (JRE), LWJGL/OpenGL native libraries, and
> ~150 MB of `.ogg` audio and texture archives. This engine is compiled straight
> to native machine code in zero-dependency Rust, with procedural audio
> synthesis and compressed `assets/textures.bin` baked directly into the `.exe`.

## Features

- **Main Menu & Screens**: Minecraft title screen with animated sky, clouds,
  hills, Creeper mascot, splash text, Singleplayer, Multiplayer, Options, and
  Pause menu.
- **Crafting & Recipe Book**: 2x2 inventory crafting and 3x3 Crafting Table
  crafting with 47 shaped recipes plus an interactive in-game **Recipes**
  browser and `FILL` helper.
- **Furnace**: Interactive furnace UI with fuel burn meter, smelting progress
  bar, and recipes for ores, cobble, sand, clay, logs, and food.
- **Redstone**: Redstone Dust wire propagation, Redstone Torches, toggleable
  Levers, Pressure Plates, and glowing Redstone Lamps.
- **Music & Sound Effects** (desktop): Pure-Rust procedural 22,050 Hz software
  mixer via Windows `winmm.dll` (`waveOut`) playing a calm looping soundtrack
  and 16 sound effects.
- **LAN Multiplayer** (desktop): Built-in TCP multiplayer (`OPEN TO LAN` /
  `JOIN SERVER` on port `25565`).
- **World, Biomes & Mobs**: 11 biomes, caves, ore veins, trees, villages, desert
  temples, mineshafts, shipwrecks (structures), 100 blocks, 50 items, Steve,
  `F5` camera cycling, and 10 mob types.

## Controls

| Key / Input | Action |
| --- | --- |
| `W A S D` | Move |
| Mouse | Look / aim |
| `Space` | Jump |
| `Shift` | Sprint |
| `F5` | Switch camera angle (1st person / 3rd person / distant) |
| `E` | Open inventory & recipes |
| `T` | Open chat |
| `1`-`9`, Mouse Wheel | Select hotbar slot |
| Left Click | Break block / attack mob / UI click |
| Right Click | Place block / use Crafting Table, Furnace, Lever / eat food |
| `Esc` | Pause menu / close UI |

## Building the native game

```powershell
cargo build --release --bin minecraft
cargo build --release --bin minecraft-setup
```

Or with `rustc` directly:

```powershell
rustc --edition 2021 -O src/main.rs -o runtime/Minecraft.exe
rustc --edition 2021 -O src/installer.rs -o runtime/Minecraft-Setup.exe
```

## Building and deploying the browser game

Prerequisites:

```bash
rustup target add wasm32-unknown-unknown
```

Build the WebAssembly library and copy it into the static site:

```bash
npm run build:wasm
```

Deploy the contents of `web/` to any static host. On Vercel, `vercel.json` sets
`web` as the output directory. The included GitHub Actions workflow
(`.github/workflows/deploy.yml`) deploys every push to `main` using the
`VERCEL_TOKEN`, `VERCEL_ORG_ID`, and `VERCEL_PROJECT_ID` repository secrets, so
you do not need to configure Vercel's Git integration manually. The
`npm run build:wasm` step runs locally, not on Vercel.

## Notes & limitations

- Software CPU renderer, typically ~15-40 FPS depending on scene complexity.
- Shipwrecks are static generated structures (not sailable ships).
- Mob AI is simplified (wandering and melee/ranged chasing).
- LAN multiplayer uses a lightweight TCP line protocol without authentication.
- The browser build is singleplayer-only and has no audio yet.
