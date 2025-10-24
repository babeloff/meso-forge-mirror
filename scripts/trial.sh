#!/bin/bash

RATTLER_CACHE_DIR=~/.cache/rattler/cache/pkgs/
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOCAL_REPO="$PROJECT_ROOT/local-conda-cache"
mkdir -p "$LOCAL_REPO"

# Use the fixed debug binary with proper OpenSSL linking
MESO_FORGE_BIN="$PROJECT_ROOT/target/debug/meso-forge-mirror"

# Build debug binary if it doesn't exist
if [ ! -f "$MESO_FORGE_BIN" ]; then
    echo "Building debug binary with system OpenSSL..."
    cd "$PROJECT_ROOT"
    PKG_CONFIG_PATH=/usr/lib64/pkgconfig:/usr/lib/pkgconfig OPENSSL_DIR=/usr OPENSSL_LIB_DIR=/usr/lib64 OPENSSL_INCLUDE_DIR=/usr/include/openssl cargo build
fi

echo "Processing openshift-installer packages..."
if [ $(find "$RATTLER_CACHE_DIR" -name "openshift-installer-4\.19\.\d+-.*\.conda" | wc -l) -lt 1 ]; then
  "$MESO_FORGE_BIN" mirror \
    --src-type zip \
    --src ~/Downloads/conda_pkgs_linux-openshift-installer.zip \
    --src-path 'conda_pkgs_linux/okd-install-4\.19\.\d+-.*\.conda' \
    --tgt-type local \
    --tgt "$LOCAL_REPO"
fi

echo "Processing coreos-installer packages..."
if [ $(find "$RATTLER_CACHE_DIR" -name "coreos-installer-0\.25\.0-.*\.conda" | wc -l) -lt 1 ]; then
  "$MESO_FORGE_BIN" mirror \
    --src-type zip \
    --src ~/Downloads/conda_pkgs_linux-coreos-installer.zip \
    --src-path 'conda_pkgs_linux/coreos-installer-0\.25\.0-.*\.conda' \
    --tgt-type local \
    --tgt "$LOCAL_REPO"
fi

echo "Processing rb-asciidoctor-diagram packages..."
if [ $(find "$RATTLER_CACHE_DIR" -name "rb-asciidoctor-diagram-3\.0\.1-.*\.conda" | wc -l) -lt 1 ]; then
  "$MESO_FORGE_BIN" mirror \
    --src-type zip \
    --src ~/Downloads/conda_pkgs_noarch-rb-asciidoctor.zip \
    --src-path 'conda_pkgs_noarch/rb-asciidoctor-diagram-3\.0\.1-.*_0\.conda' \
    --tgt-type local \
    --tgt "$LOCAL_REPO"
fi

echo "Processing rb-asciidoctor-revealjs packages..."
if [ $(find "$RATTLER_CACHE_DIR" -name "rb-asciidoctor-revealjs-5\.2\.0-.*\.conda" | wc -l) -lt 1 ]; then
  "$MESO_FORGE_BIN" mirror \
    --src-type zip \
    --src ~/Downloads/conda_pkgs_noarch-rb-asciidoctor.zip \
    --src-path 'conda_pkgs_noarch/rb-asciidoctor-revealjs-5\.2\.0-.*_0\.conda' \
    --tgt-type local \
    --tgt "$LOCAL_REPO"
fi

echo "Final package placement verification:"
find "$LOCAL_REPO" -name "*.conda" | sort
