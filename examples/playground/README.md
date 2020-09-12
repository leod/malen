# Playground Example

## How to Build
Install dependencies:
```
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

Build:
```
wasm-pack build --target web --no-typescript
cp pkg/{playground.js,playground_bg.wasm} static/
```

