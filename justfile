# Flashes a Pico 2 connected in BOOTSEL/picoboot mode (needs picotool installed).
run:
    mkdir -p build
    DEOR_LIB=lib deor main.deor build/main.rs
    cargo run

build:
    mkdir -p build
    DEOR_LIB=lib deor main.deor build/main.rs
    cargo build

release:
    mkdir -p build
    DEOR_LIB=lib deor main.deor build/main.rs
    cargo build --release

# Builds the RP2040 (Pico 1) stand-in used to simulate this project in
# Wokwi — Wokwi has no RP2350 core yet. Not what ships to real hardware.
wokwi-build:
    mkdir -p build
    DEOR_LIB=lib deor main.deor build/main.rs
    cargo build --no-default-features --features rp2040 --target thumbv6m-none-eabi

run-spec:
    cd deor_specification && bun server.js

update-deor-with-latest:
    curl -sSf https://raw.githubusercontent.com/nathanphoffman/DeorLang/main/setup/update.sh | sh

install-deor:
    curl -sSf https://raw.githubusercontent.com/nathanphoffman/DeorLang/main/setup/install-deor.sh | sh

install-ext:
    #!/bin/sh
    TMP="$(mktemp -d)"
    curl -sL "https://github.com/nathanphoffman/DeorLang/archive/refs/heads/main.tar.gz" | tar xz -C "$TMP"
    code --install-extension "$(ls "$TMP/DeorLang-main/deor-vscode/"*.vsix | tail -1)"
    rm -rf "$TMP"
    echo "Done — reload VS Code window to apply (Ctrl+Shift+P → Reload Window)."

