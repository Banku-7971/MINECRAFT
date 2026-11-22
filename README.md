# Sigma Craft

**by SIGMA BANKU**

## Play the standalone Windows game

Download `Sigma-Craft-Windows.zip` from the releases page, extract it, and open `Sigma-Craft.exe` on 64-bit Windows. Click the game window to play. This is a separate, offline creative block sandbox; it needs no Minecraft installation, account, Java, or Rust. It is not full Minecraft and does not have survival, crafting, mobs, multiplayer, or Minecraft-world compatibility. Controls and save location are in [standalone/README.md](standalone/README.md).

## Original client source

Sigma Craft is a Rust-based Minecraft: Java Edition client, branded by SIGMA BANKU. It is a fork, not a new implementation; its original source and attribution are listed in Credits and license below.

## Build and play

1. Restore the bundled upstream source, assets, documentation, tools, tests and benchmarks **before building**:
   ```sh
   ./restore-upstream-support.sh
   ```
   Some upstream source and support directories are bundled in `upstream-support.tar.gz` and ignored after extraction to keep the checkout manageable. The script refuses to overwrite existing directories. To distribute changes to those files, update the archive too.
2. Install Rust **1.95 or newer** and the platform-specific build dependencies described in `docs/native-client.md` and `launcher/README.md` after restoration. Resource preparation also requires JDK 25.
3. You need your **own copy of Minecraft: Java Edition 26.3** and its assets. Follow `docs/native-client.md` to prepare the resources and build the game, or build the launcher with `cd launcher && cargo build --release`.

The launcher still uses the upstream project's release, account and other online services. Its normal downloads are **upstream binaries**, not the branded game built from this source. To play this fork through the launcher, build the game locally and use its **Import a game build…** feature as described in `launcher/README.md`. Upstream downloads may still show their original branding. Existing settings, save paths, APIs, and some interface text retain upstream identifiers for compatibility.

## Upstream automation

The upstream GitHub Actions workflows are preserved in `.github/upstream-workflows/` but disabled in this fork. They target upstream infrastructure and require review and adaptation before use.

## Credits and license

Sigma Craft is a fork branded by SIGMA BANKU, based on [Horizon](https://github.com/horizon-rust/horizon) commit `2b13c0624036bcbd2983f41d86889398e2cd889d`, built by the Horizon Team and contributors. See [UPSTREAM_README.md](UPSTREAM_README.md) for their original credits and project information. Horizon's original work is licensed under [GNU LGPL 3.0 or later](LICENSE); earlier MIT-licensed work and dependency notices are retained in [LICENSES](LICENSES) and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

NOT AN OFFICIAL MINECRAFT PRODUCT. NOT APPROVED BY OR ASSOCIATED WITH MOJANG OR MICROSOFT. Minecraft is a trademark of Mojang Synergies AB; its code and assets belong to Mojang and Microsoft and are not included here.
