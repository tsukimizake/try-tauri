set shell := ["nu", "-c"]

test:
    cargo test --manifest-path tauri/Cargo.toml

dev:
    bunx tauri dev

elm_new_file path:
    echo {{ path }}| path dirname | mkdir $"src/elm/($in)"
    echo {{ path }} | path split | str join "." | str substring 0..-5 | $"module ($in) exposing \(..\)\nhoge = identity" | save $"src/elm/{{ path }}"

setem:
    bunx setem
    mv RecordSetter.elm src/elm/RecordSetter.elm

just_format:
    just --fmt --unstable
