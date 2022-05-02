# WAW - WebAssembly Windows

A window manager written in rust + yew compiled to web assembly.

## Usage
Import the javascript gloo module
```javascript
import init, {Screen} from "./waw.js"; // Static (Inside another module)

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

Create and destroy windows
```javascript
const windowId = await screen.newWindow();
screent.deleteWindow(windowId);
```

Each window is identified by an id. To get the window's `<div>` to add content to use
```javascript
const div = screen.getWindow(windowId);
div.innerHTML = "Hello World!";
```

See `example/index.html` for more.

