# Minecraft (Rust Edition)

A native Windows desktop voxel game written from scratch in **Rust** with zero external crate dependencies.

## Windows Executables

Built binaries in `runtime/`:
- **`runtime/Minecraft-Setup.exe`** — Native Windows GUI installer. Installs `Minecraft.exe` to `%LOCALAPPDATA%\Minecraft\`, creates Desktop & Start Menu shortcuts, registers **Minecraft** in Windows Apps & Features (with uninstall support), and offers to launch immediately.
- **`runtime/Minecraft.exe`** — Standalone portable Windows 64-bit GUI application (`x86_64-pc-windows-gnu`), zero external DLL dependencies.

## Features

- **Main Menu & Screens**: Minecraft title screen with animated sky, clouds, hills, Creeper mascot, splash text, Singleplayer, Multiplayer, Options, and Pause menu.
- **Crafting & Recipe Book**: 2×2 inventory crafting and 3×3 Crafting Table crafting with 47 shaped recipes plus an interactive in-game **Recipes** browser and `FILL` helper.
- **Furnace**: Interactive furnace UI with fuel burn meter, smelting progress bar, and recipes for ores, cobble, sand, clay, logs, and food.
- **Redstone**: Redstone Dust wire propagation, Redstone Torches, toggleable Levers, Pressure Plates, and glowing Redstone Lamps.
- **Music & Sound Effects**: Pure-Rust procedural 22,050 Hz software mixer via Windows `winmm.dll` (`waveOut`) playing a calm looping soundtrack and 16 sound effects (clicks, footsteps, breaking, placing, crafting, furnace, hurt, mobs, explosions).
- **LAN Multiplayer**: Built-in TCP multiplayer (`OPEN TO LAN` / `JOIN SERVER` on port `25565`) syncing player positions, block edits, and live `T` chat.
- **World, Biomes & Mobs**: 11 biomes, caves, ore veins, trees, villages, desert temples, mineshafts, shipwrecks (structures), 100 blocks, 50 items, Minecraft Steve character, `F5` camera cycling (first-person / third-person / distant), and 10 mob types.

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
| `1`–`9`, Mouse Wheel | Select hotbar slot |
| Left Click | Break block / attack mob / UI click |
| Right Click | Place block / use Crafting Table, Furnace, Lever / eat food |
| `Esc` | Pause menu / close UI |

## Building from Source on Windows

```powershell
cargo build --release --bin minecraft
cargo build --release --bin minecraft-setup
```

Or with `rustc` directly:

```powershell
rustc --edition 2021 -O src/main.rs -o runtime/Minecraft.exe
rustc --edition 2021 -O src/installer.rs -o runtime/Minecraft-Setup.exe
```

## Notes & Limitations

- Software CPU renderer (`480×270` scaled to the window via GDI `StretchDIBits`), typically ~15–30 FPS depending on scene complexity.
- Shipwrecks are static generated structures (not sailable ships).
- Mob AI is simplified (wandering and melee/ranged chasing).
- LAN multiplayer uses a lightweight TCP line protocol without authentication.
