<div align="center">
    <h1><code>jumpdrive</code></h1>

[<img alt="github" src="https://img.shields.io/badge/UBC--iGEM-jumpdrive-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="22">](https://github.com/UBC-iGEM/jumpdrive)
[<img alt="docs" src="https://img.shields.io/github/deployments/UBC-iGEM/jumpdrive/production?label=docs&style=for-the-badge&labelColor=555555&logo=docs.rs" height="22">](http://ubcigem.com/jumpdrive/jumpdrive/index.html)

</div>
<div align="center">
    <p><strong>A minimal, <em>🚀 blazingly fast</em> HTTP server with websocket support</strong></p>
    <h3>
        <a href="http://ubcigem.com/jumpdrive/jumpdrive/index.html">Docs</a>
        <span> | </span>
        <a href="https://ubcigem.com/">UBC iGEM</a>
    </h3>
    <br/>
</div>


## Overview
Jumpdrive is a slim, minimally-featured library to statically serve a directory over HTTP. It supports extensibility for custom `GET` endpoints and a WebSocket (RFC 6455) connection.

## Usage
Jumpdrive executes its event loop via the titular [`jumpdrive!`](https://ubcigem.com/jumpdrive/jumpdrive/macro.jumpdrive.html) macro, and provides various helper functions to assist endpoint handlers.
```rust
jumpdrive! {
        dir = "public/",
        ws = "/ws": websocket_handler,
        routes = {
                "/csv": csv_server
        },
        err = error_handler
}
```

## Docs
To read the full documentation, visit the doc page [here](http://ubcigem.com/jumpdrive/jumpdrive/index.html).
