test:
    cargo test --manifest-path tauri/Cargo.toml

dev:
    bunx tauri dev

just_format:
    just --fmt --unstable
