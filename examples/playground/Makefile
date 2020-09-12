build:
	wasm-pack build --target web --no-typescript
	cp static/* pkg/

run: build
	(cd pkg && python3 -m http.server)

.PHONY: build run
