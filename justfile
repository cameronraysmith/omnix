default:
    @just --list

# Auto-format the source tree
fmt:
    find crates/omnix-gui/src -name \*.rs | xargs -n1 sh -c 'echo "📔 $1"; dx fmt -f $1' sh
    # Run treefmt *after* 'dx fmt' because the later rewrites the former!
    treefmt

alias f := fmt

# CI=true for https://github.com/tauri-apps/tauri/issues/3055#issuecomment-1624389208)
bundle $CI="true":
    # HACK (change PWD): Until https://github.com/DioxusLabs/dioxus/issues/1283
    cd ./crates/omnix-gui/assets && dx bundle --release
    nix run nixpkgs#lsd -- --tree ./dist/bundle/macos/omnix-gui.app

# Run omnix-gui locally
watch-gui $RUST_BACKTRACE="1":
    # XXX: hot reload doesn't work with tailwind
    # dx serve --hot-reload
    cd ./crates/omnix-gui && dx serve --bin omnix-gui

alias wg := watch-gui

# Run omnix-cli locally
watch *ARGS:
    cargo watch -p omnix-cli -x 'run -- {{ARGS}}'

alias w := watch

# Run tests
test:
    cargo test

test-cli:
    cd ./crates/omnix-cli && cargo watch -x test

# Run docs server (live reloading)
doc-cargo:
    cargo-doc-live

# Run mdbook live server
doc:
    cd ./doc && mdbook serve

# Do link checks on docs
doc-check:
    nix run .#doc-linkCheck

# Run CI locally
ci:
    nix run . ci

clippy:
    cargo clippy --release --locked --all-targets --all-features -- --deny warnings

# Setup node_modules using Nix (invoked automatically by nix-shell)
node_modules NODE_PATH:
    rm -f ./e2e/node_modules
    ln -sf ${NODE_PATH} ./e2e/node_modules
