#!/bin/bash

multicompress() {
    local file="$1"
    if command -v gzip &>/dev/null; then
        gzip --best --stdout --keep "$file" >"$file.gz"
    fi

    if command -v zstd &>/dev/null; then
        zstd --keep --force -19 --quiet "$file" -o "$file.zst"
    fi

    if command -v brotli &>/dev/null; then
        brotli --best --force -o "$file.br" "$file"
    fi
}

export -f multicompress
# create pre-compressed variants gzip, zstd, brotli for dist files
find ./dist/ -type f ! -name '*.gz' ! -name '*.br' ! -name '*.zst' -exec bash -c 'multicompress "$0"' {} \;
