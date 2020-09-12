build:
	wasm-pack build --target web --no-typescript
	cp pkg/playground.js pkg/playground_bg.wasm static/

run: build
	(cd static && python3 -m http.server)

.PHONY: build run
