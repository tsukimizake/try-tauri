set shell := ["nu", "-c"]

test:
    cargo test --manifest-path tauri/Cargo.toml

elm_compile:
  npx elm make src/elm/Main.elm | ignore

dev:
    bunx tauri dev

elm_new_file path:
    echo {{ path }}| path dirname | mkdir $"src/elm/($in)"
    echo {{ path }} | path split | str join "." | str substring 0..-5 | $"module ($in) exposing \(..\)\nhoge = identity" | save $"src/elm/{{ path }}"

setem:
    bunx setem
    mv RecordSetter.elm src/generated/RecordSetter.elm

lint:
    npx elm-review --fix

just_format:
    just --fmt --unstable
