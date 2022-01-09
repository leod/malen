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

Then open http://localhost:8080/ in a browser.

### Step by step
If the `Makefile` does not work for you, you can run the steps manually.

```
wasm-pack build --target web --no-typescript ; cp static/* pkg/ ; python3 server.py pkg

### Windows Setup

The `openssl-sys` dependency of `wasm-pack` can be quite a pain on Windows.

There are many different installation instructions flying around. This one actually worked for me
(for `openssl-sys v0.9.72`):
1. Install `Win64 OpenSSL v1.1.1m` from http://slproweb.com/products/Win32OpenSSL.html to `C:\OpenSSL-Win64`.
2. Download `cacert.pem` to `C:\OpenSSL-Win64\certs` (creating a new directory `certs`).
3. In PowerShell, set environment variables as follows:
   ```
   $env:OPENSSL_DIR="C:\OpenSSL-Win64"
   $env:SSL_CERT_FILE="C:\OpenSSL-Win64\certs\cacert.pem"
   $env:OPENSSL_NO_VENDOR=1
   ```
   Then run:
   ```
   cargo install wasm-pack --target-dir tmp-wasm-pack-cache
   ```
   (the cache directory is not needed, but allows for faster iteration if compilation fails.)

## Resource Credits

- `Ground_*.png`

   Author: Cethiel

   URL: https://opengameart.org/content/tileable-bricks-ground-textures-set-1

- `Brick_*.png`

   Author: Cethiel

   URL: https://opengameart.org/content/tileable-brick-ground-textures-set-2