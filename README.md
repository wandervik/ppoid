# PPOid :rocket: :rocket:

## Setup

Install rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Add wasm target:
```bash
rustup target add wasm32-unknown-unknown
```

Install wasm bindgen
```bash
cargo install wasm-bindgen-cli
```


## Build
Build app:
```bash
cargo build --release --target wasm32-unknown-unknown
```
Generate wasm bindings and place them into `out` dirrectory:
```bash
wasm-bindgen --out-name ppoid --out-dir out --target web ./target/wasm32-unknown-unknown/release/ppoid.wasm
```
Copy content of the static into `out` dirrectory
```bash
cp -a ./static/. ./out/
```

Serve content of the `out` directory
