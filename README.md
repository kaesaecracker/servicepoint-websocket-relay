# ServicePoint WebSocket Relay

This is a simple WebSocket server for forwarding messages to protocols not supported in the browser.

## Build

```shell
cargo build --release
```

## Running

All configuration is passed on the command line.

```shell
target/release/servicepoint-websocket-relay --help

# Usage: servicepoint-websocket-relay [OPTIONS]
# 
# Options:
#  -l, --listen <LISTEN>    The address to listening on [default: 127.0.0.1:8080]
#  -f, --forward <FORWARD>  The address to forward to [default: 127.0.0.1:2342]
#  -h, --help               Print help
#  -V, --version            Print version
```
