# Sigma Craft

**by SIGMA BANKU**

Sigma Craft is a branded fork of [Horizon](https://github.com/horizon-rust/horizon), a Rust reimplementation of Minecraft: Java Edition. It is based on Horizon commit `2b13c0624036bcbd2983f41d86889398e2cd889d`. The launcher and game startup screens carry the Sigma Craft name; the underlying project, package names and compatibility identifiers remain Horizon's. This is not a new implementation of Minecraft and has the same capabilities and limitations as that Horizon revision.

## Build and play

1. Restore the bundled upstream source, assets, documentation, tools, tests and benchmarks **before building**:
   ```sh
   ./restore-upstream-support.sh
   ```
   Some upstream source and support directories are bundled in `upstream-support.tar.gz` and ignored after extraction to keep the checkout manageable. The script refuses to overwrite existing directories. To distribute changes to those files, update the archive too.
2. Install Rust **1.95 or newer** and the platform-specific build dependencies described in `docs/native-client.md` and `launcher/README.md` after restoration. Resource preparation also requires JDK 25.
3. You need your **own copy of Minecraft: Java Edition 26.3** and its assets. Follow `docs/native-client.md` to prepare the resources and build the game, or build the launcher with `cd launcher && cargo build --release`.

The launcher still uses Horizon's release, account and other online services. Its normal downloads are **upstream Horizon binaries**, not the branded game built from this source. To play this fork through the launcher, build the game locally and use its **Import a game build…** feature as described in `launcher/README.md`. Upstream updates or downloaded binaries may still display Horizon branding. Existing Horizon settings, save paths, APIs, assets, and some interface text also retain the Horizon name.

## Upstream automation

Horizon’s GitHub Actions workflows are preserved in `.github/upstream-workflows/` but disabled in this fork. They target upstream infrastructure and require review and adaptation before use.

## Credits and license

Sigma Craft is a fork branded by SIGMA BANKU, built upon the work of the Horizon Team and its contributors. See [UPSTREAM_README.md](UPSTREAM_README.md) for their original credits and project information. Horizon's original work is licensed under [GNU LGPL 3.0 or later](LICENSE); earlier MIT-licensed work and dependency notices are retained in [LICENSES](LICENSES) and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

NOT AN OFFICIAL MINECRAFT PRODUCT. NOT APPROVED BY OR ASSOCIATED WITH MOJANG OR MICROSOFT. Minecraft is a trademark of Mojang Synergies AB; its code and assets belong to Mojang and Microsoft and are not included here.
