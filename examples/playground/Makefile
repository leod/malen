build:
	wasm-pack build --target web --no-typescript --dev
	cp static/* pkg/

build-release:
	wasm-pack build --target web --no-typescript --release
	cp static/* pkg/

run: build
	(cd pkg && python3 -m http.server)

run-release: build-release
	(cd pkg && python3 -m http.server)

.PHONY: build build-release run run-release
