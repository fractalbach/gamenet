## Procede

Procedural world generation sub-project.

### Setting Up Rust:

* Install rust (`curl https://sh.rustup.rs -sSf | sh`)
* Install wasm target (Ex: `rustup target add wasm32-unknown-unknown`)
* Install wasm-bindgen-cli (Ex: `cargo install wasm-bindgen-cli`)
    * If you get:
        "error: failed to run custom build command for 'openssl-sys v*'":
        Install libssl-dev (Ex: `apt install libssl-dev`)
