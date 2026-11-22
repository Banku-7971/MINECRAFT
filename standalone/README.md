# Sigma Craft: standalone Windows sandbox

A separate, original creative block sandbox by SIGMA BANKU. It does not need Minecraft, a Minecraft account, Java, Rust, or network access. It is **not full Minecraft**: this first version has a small finite generated world, walking, jumping, mining, placing nine kinds of blocks and automatic world saving. It has no survival mode, crafting, mobs, multiplayer or Minecraft-world compatibility.

## Play

Download `Sigma-Craft-Windows.zip`, extract it, and double-click `Sigma-Craft.exe` on 64-bit Windows. The executable is unsigned. Click inside the window to capture the mouse.

- **W A S D** move, **mouse** look, **Space** jump, **Ctrl** run
- **Left click** breaks a block; **right click** places one
- **1–9** or the mouse wheel selects a block
- **Esc** releases the mouse and pauses; click again to resume
- **F5** saves; the game also saves changed blocks automatically and on exit

The world is kept in `%APPDATA%\SigmaCraft\world.dat`, not beside the executable. Back up that file to keep your creations. If it cannot be read, saving is disabled rather than overwriting it.

## Building from source

For maintainers: use Zig's C compiler (the tested version is `0.12.0-dev.1286`) from a Linux or Windows host, with the Windows x64 target:

```sh
cd standalone
zig rc /fo sigma.res sigma.rc
zig cc -target x86_64-windows-gnu -O2 -std=c11 main.c world.c sigma.res -o Sigma-Craft.exe -Wl,--subsystem,windows -lgdi32 -luser32 -lm
```

Portable world logic tests: `cc -std=c11 -O2 world.c world_test.c -lm -o world_test && ./world_test`.
