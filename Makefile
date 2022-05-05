all: compile bundle

compile:
	wasm-pack build --target web

bundle:
	mkdir -p build
	cp static/waw.css build/
	cp pkg/waw.js build/
	cp pkg/waw_bg.wasm build/
