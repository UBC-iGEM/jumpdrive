<div align="center">
    <h1><code>jumpdrive</code></h1>
    <a href="https://github.com/UBC-iGEM/jumpdrive" style="text-decoration: none;">
        <img alt="github" src="https://img.shields.io/badge/UBC--iGEM-jumpdrive-8da0cb?style=for-the-badge&labelColor=555555&logo=github">
    </a>
    <a href="https://jumpdrive.vercel.app/jumpdrive/" style="text-decoration: none;">
        <img alt="github" src="https://img.shields.io/github/deployments/UBC-iGEM/jumpdrive/production?&style=for-the-badge&labelColor=555555&logo=vercel">
    </a>
    <br/>
    <br/>
    <p><strong>A minimal, <em>🚀 blazingly fast</em> HTTP server with websocket support</strong></p>
    <h3>
        <a href="https://jumpdrive.vercel.app/jumpdrive/">Docs</a>
        <span> | </span>
        <a href="https://ubcigem.com/">UBC iGEM</a>
    </h3>
</div>

## Overview
Jumpdrive is a slim, minimally-featured library to statically serve a directory over HTTP. In addition, handling of one or more `GET` endpoints and a WebSocket (RFC 6455) connection is supported.

## Usage
Jumpdrive executes its event loop via the titular `jumpdrive!` macro, and provides various helper functions to assist endpoint handlers.
```rust
jumpdrive! {
    dir = "public/",
    ws = ["/ws", ws_handler],
    routes = {
        "/csv": csv_server
    },
    err = err_callback
}
```

## Docs
To read the full documentation, visit the doc page [here](https://jumpdrive.vercel.app/jumpdrive/).
