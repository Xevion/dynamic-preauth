#!/bin/bash
set -e

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

    # TODO: Add deflate
}

commas() {
    sed ':a;s/\B[0-9]\{3\}\>/,&/;ta'
}

get_size() {
    find ./dist/ -type f -name $1 -print0 | du --files0-from=- -bc | tail -n1 | awk '{print $1}' | commas
}

export -f multicompress

# find only non-compressed files in dist folder
FILES=$(find ./dist/ -type f ! -name '*.gz' ! -name '*.br' ! -name '*.zst')

# create pre-compressed variants gzip, zstd, brotli for dist files
echo "$FILES" | xargs -n1 -P0 bash -c 'multicompress "$@"' _

# calculate sizes
ORIGINAL_SIZE=$(echo "$FILES" | tr '\n' '\0' | du --files0-from=- -bc | tail -n1 | awk '{print $1}' | commas)
GZIP_SIZE=$(get_size '*.gz')
ZSTD_SIZE=$(get_size '*.zst')
BROTLI_SIZE=$(get_size '*.br')

export LC_NUMERIC="C.utf8"
printf "Original size: %s bytes\n" $ORIGINAL_SIZE
printf "Gzip size: %s bytes\n" "$GZIP_SIZE"
printf "Zstd size: %s bytes\n" "$ZSTD_SIZE"
printf "Brotli size: %s bytes\n" "$BROTLI_SIZE"
