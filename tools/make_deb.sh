#!/bin/bash

# Configuration
PACKAGE_NAME="mistral-cli"
VERSION="0.1.1"
ARCHITECTURE="amd64"
MAINTAINER="noskla <m-cli.unreached633@passinbox.com>, aumbriac"
DESCRIPTION="mistral-cli - Rust CLI for Mistral AI API"
BUILD_DIR="build"
PACKAGE_DIR="$BUILD_DIR/$PACKAGE_NAME"
DEBIAN_DIR="$PACKAGE_DIR/DEBIAN"
# -----------------------

mkdir -p "$DEBIAN_DIR"

echo "Building Rust project..."
cargo build --release || { echo "Build failed"; exit 1; }

echo "Creating package structure..."
mkdir -p "$PACKAGE_DIR/usr/local/bin"
mkdir -p "$PACKAGE_DIR/etc/mistral-cli"

cp "target/release/mistral" "$PACKAGE_DIR/usr/local/bin/"

cat > "$PACKAGE_DIR/etc/mistral-cli/config.json" << EOF
{
  "api_key": null,
  "preferred_model": "mistral-tiny"
}
EOF

cat > "$DEBIAN_DIR/control" << EOF
Package: $PACKAGE_NAME
Version: $VERSION
Section: utils
Priority: optional
Architecture: $ARCHITECTURE
Maintainer: $MAINTAINER
Description: $DESCRIPTION
EOF

echo "Building Debian package..."
dpkg-deb --build "$PACKAGE_DIR" || { echo "Package build failed"; exit 1; }

echo "Package built successfully: ${PACKAGE_NAME}_${VERSION}_${ARCHITECTURE}.deb"
