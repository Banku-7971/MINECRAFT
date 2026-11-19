# RustCraft

A Minecraft-inspired voxel sandbox whose world, renderer, textures, lighting and entities are all written in **Rust**. A tiny Node.js viewer streams the frames the Rust engine draws, so the game is playable in the browser without any graphics crates.

## Play

```bash
./build.sh          # needs rustc (or set RUSTC=/path/to/rustc)
./run.sh            # starts the engine and viewer on http://localhost:3000
```

`run.sh` rebuilds automatically when `src/main.rs` is newer than the binary. `npm start` does the same thing.

## Controls

| Key | Action |
| --- | --- |
| `W A S D` | Move |
| Mouse | Look |
| `Space` | Jump |
| `Shift` | Sprint |
| `F5` | First person / third person / distant camera |
| `E` | Creative inventory |
| `1`–`9`, wheel | Hotbar slot |
| Left click | Break block / attack mob |
| Right click | Place block |

## What is implemented

- Procedural chunked world with plains, forest, desert, savanna, taiga, snowy tundra, jungle, swamp, mountains, beaches and oceans.
- Caves, ore veins, water, lava, trees, cactus, flowers, farmland, mineshafts, villages, desert temples and shipwrecks.
- 95 block types and 50 items, including tools, food and resources, all selectable from the creative inventory.
- Pixel-art textures generated in Rust (32×32 per material), face shading, ambient occlusion, fog, animated water/lava, clouds, a day/night cycle and sun/moon.
- Steve player sprite with two third-person camera modes, plus sheep, cows, pigs, chickens, villagers, zombies, skeletons, creepers, spiders and endermen with wandering/chasing AI and drops.
- Hotbar, health/hunger HUD, block breaking/placing, mob combat, item pickups and respawning.

This is a compact homage to Minecraft rather than a copy of the full vanilla game: crafting, redstone, sound and multiplayer are not included.
