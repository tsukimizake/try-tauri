set shell := ["nu", "-c"]

test:
    cargo test --manifest-path tauri/Cargo.toml

dev:
    bunx tauri dev

elm_new_file path:
    echo {{ path }}| path dirname | mkdir $"src/elm/($in)"
    touch $"src/elm/{{ path }}"

setem:
    bunx setem
    mv RecordSetter.elm src/elm/RecordSetter.elm

just_format:
    just --fmt --unstable

