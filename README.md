# WAW - WebAssembly Windows

A window manager written in rust + yew compiled to web assembly.

## Compiling
Make sure you have [wasm-pack](https://github.com/rustwasm/wasm-pack) installed
```bash
cargo install wasm-pack
```

And execute it
```bash
wasm-pack build --target web
```

## Usage
Attach the required css file
```html
<link rel="stylesheet" href="/static/waw.css">
```

Import the javascript gloo module
```javascript
import init, {Screen} from "/pkg/waw.js"; // Static (Inside another module)

const {default: init, Screen} = await import("./waw.js"); // Asynchronous
```

Run the `init` function
```javascript
await init();
```

Create a `Screen` inside an html container element (currently only `<body>` is really supported)
```javascript
const screen = new Screen(document.body);
```

Register a window
```javascript
const windowId = await screen.newWindow({title: "Some Window", dock: "left"});
```

See `example/index.html` or [pnp-zone](https://github.com/pnp-zone/docs/blob/main/docs/dev/plugins/waw.md) for more.

## Styling

For now see `example/static/pnp-zone.css` as ref.

