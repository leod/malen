build:
	wasm-pack build --target web --no-typescript --dev
	cp static/* pkg/

build-release:
	wasm-pack build --target web --no-typescript --release
	cp static/* pkg/

run: build
	python3 server.py pkg/

run-release: build-release
	python3 server.py pkg/

.PHONY: build build-release run run-release
