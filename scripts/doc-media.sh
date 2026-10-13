#!/usr/bin/env bash
# Renders the documentation's screenshots and animations from the app and
# encodes them into website/public/media/<language>/ (mise run doc:media).
#
# Usage: doc-media.sh [SECTION]
#   SECTION: one documentation section (getting-started, vehicles, fleet,
#            workshop, brand, export, ai-assistant); all of them when empty.
#            Media of the other sections are left untouched.
#
# Needs a GPU (the app renders with wgpu) and ffmpeg (installed by the mise
# task). Screenshots become WebP images, animations H.264 MP4 videos with a
# WebP poster (their first frame). Also regenerates the site's color tokens
# and fonts, and writes website/public/media/manifest.json.

set -euo pipefail

section="${1:-}"
root="$(cd "$(dirname "$0")/.." && pwd)"
raw="$root/target/doc-media"
media="$root/website/public/media"
languages=(en fr es de)

if ! command -v ffmpeg > /dev/null; then
    echo "ffmpeg is needed to encode the media: run this through 'mise run doc:media'." >&2
    exit 1
fi

echo "== Color tokens and fonts"
cargo run -q --manifest-path "$root/Cargo.toml" -p tp-ui --example tokens_css
for font in Geist-Regular Geist-Medium Geist-SemiBold JetBrainsMono-Regular JetBrainsMono-Medium; do
    cp "$root/assets/fonts/$font.ttf" "$root/website/src/fonts/"
done
cp "$root/assets/fonts/Geist-LICENSE.txt" "$root/assets/fonts/JetBrainsMono-LICENSE.txt" "$root/website/src/fonts/"

echo "== Rendering ${section:-every section}"
if [ -n "$section" ]; then
    for lang in "${languages[@]}"; do
        rm -rf "${raw:?}/$lang/$section"
    done
    filter=(--exact "section_${section//-/_}")
else
    rm -rf "$raw"
    filter=(section_)
fi
cargo test -q --manifest-path "$root/Cargo.toml" -p tp-app --test doc_media -- \
    --ignored --test-threads=1 "${filter[@]}"

echo "== Encoding"
for lang in "${languages[@]}"; do
    src="$raw/$lang${section:+/$section}"
    dst="$media/$lang${section:+/$section}"
    rm -rf "$dst"
    [ -d "$src" ] || continue
    # Screenshots: every PNG outside an animation's frame folder.
    while IFS= read -r png; do
        rel="${png#"$raw/$lang/"}"
        out="$media/$lang/${rel%.png}.webp"
        mkdir -p "$(dirname "$out")"
        ffmpeg -nostdin -loglevel error -y -i "$png" -c:v libwebp -quality 85 "$out"
    done < <(find "$src" -name '*.png' ! -name 'frame_*.png')
    # Animations: every folder of numbered frames.
    while IFS= read -r first; do
        dir="$(dirname "$first")"
        rel="${dir#"$raw/$lang/"}"
        out="$media/$lang/$rel"
        mkdir -p "$(dirname "$out")"
        ffmpeg -nostdin -loglevel error -y -framerate 30 -i "$dir/frame_%05d.png" \
            -c:v libx264 -preset slow -crf 22 -pix_fmt yuv420p -movflags +faststart -an \
            "$out.mp4"
        ffmpeg -nostdin -loglevel error -y -i "$first" -c:v libwebp -quality 85 "$out.poster.webp"
    done < <(find "$src" -name 'frame_00001.png')
done

echo "== Manifest"
version="$(sed -n 's/^version = "\(.*\)"/\1/p' "$root/Cargo.toml" | head -1)"
commit="$(git -C "$root" rev-parse HEAD)"
if [ -n "$(git -C "$root" status --porcelain -- ':!website/public/media' ':!website/src/styles/tokens.css')" ]; then
    dirty=true
else
    dirty=false
fi
{
    printf '{\n  "version": "%s",\n  "commit": "%s",\n  "dirty": %s,\n  "generated": "%s",\n  "files": {\n' \
        "$version" "$commit" "$dirty" "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    for i in "${!languages[@]}"; do
        lang="${languages[$i]}"
        printf '    "%s": [' "$lang"
        if [ -d "$media/$lang" ]; then
            (cd "$media/$lang" && find . -type f | sed 's|^\./||' | LC_ALL=C sort) \
                | awk 'NR > 1 { printf ", " } { printf "\"%s\"", $0 }'
        fi
        printf ']'
        [ "$i" -lt $((${#languages[@]} - 1)) ] && printf ','
        printf '\n'
    done
    printf '  }\n}\n'
} > "$media/manifest.json"

du -sh "$media" | awk '{ print "== Done: " $1 " of media in website/public/media" }'
