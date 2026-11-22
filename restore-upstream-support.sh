#!/bin/sh
# Restore the unmodified Horizon support files and source directories needed by the build.
set -eu
cd "$(dirname "$0")"
for dir in docs tools tests benchmarks launcher/lang launcher/docs mods third_party \
    crates/horizon-client/src/game/modding crates/horizon-client/src/game/network \
    crates/horizon-client/src/game/blocks crates/horizon-client/src/game/mobs \
    crates/horizon-client/src/game/entities crates/horizon-client/src/game/administration \
    crates/horizon-client/src/game/world crates/horizon-client/src/game/items \
    crates/horizon-client/src/game/native_hooks; do
    if [ -e "$dir" ]; then
        echo "Refusing to overwrite existing $dir. Move it aside before restoring." >&2
        exit 1
    fi
done
tar -xzf upstream-support.tar.gz
echo 'Restored upstream files. They are ignored by Git; edit the archive to redistribute changes to them.'
