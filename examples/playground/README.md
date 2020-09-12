# Playground Example

## How to Build
Install dependencies:
```
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

Build and run using the [`Makefile`](Makefile):
```
make run
```

### Step by step
If the `Makefile` does not work for you, you can run the steps manually.

Build:
```
wasm-pack build --target web --no-typescript
cp pkg/{playground.js,playground_bg.wasm} static/
```

Run:
```
(cd static && python3 -m http.server)
```
(or using any other HTTP server)
