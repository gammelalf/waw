# WAW - WebAssembly Windows

A window manager written in rust + yew compiled to web assembly.

## Usage
Attach the required css file
```html
<link rel="stylesheet" href="/static/functional.css">
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
const windowId = await screen.newWindow({title: "Some Window"});
```

Each window is identified by an id. To get the window's `<div>` to add content to use
```javascript
const div = await screen.getWindow(windowId);
div.innerHTML = "Hello World!";
```

See `example/index.html` for more.

