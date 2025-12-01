# #!/usr/bin/env bash

# This script copies the release executable of ctftools and
# generates a SHA256 checksum of the executable to be uploaded
# to the GitHub workflow artifacts.
#
# This script is meant to be used in GitHub Actions.

if [ $# -lt 4 ]; then
    echo "Usage: $0 <MATRIX_HOST> <MATRIX_TARGET> <MATRIX_LABEL> <VERSION>"
    exit 1
fi

MATRIX_HOST="$1"
MATRIX_TARGET="$2"
MATRIX_LABEL="$3"
VERSION="$4"

# Determine executable extension for Windows
BIN_EXT=""
if [ "$MATRIX_HOST" = "windows" ]; then
    BIN_EXT=".exe"
fi

# Prepare workspace
mkdir -p "target/tmp"

SRC="target/$MATRIX_TARGET/release/ctftools$BIN_EXT"
DEST="target/tmp/ctftools-$VERSION-$MATRIX_LABEL$BIN_EXT"
CHECKSUM_FILE="target/tmp/ctftools-$VERSION-$MATRIX_LABEL$BIN_EXT.sha256"

# Validate source exists
if [ ! -f "$SRC" ]; then
    echo "Error: execution file not found: $SRC"
    exit 1
fi

# Copy artifact
cp "$SRC" "$DEST"

# To trim the target/tmp/ part
cd $(dirname $DEST)

# Generate SHA256 checksum
sha256sum "$DEST" > "$CHECKSUM_FILE"

# Go back to the previous directory then done
cd -
