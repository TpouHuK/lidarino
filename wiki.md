# Documentation
`./build_docs.sh --open` for building docs and opening them, or
`./build_docs.sh` for just building the docs.


# `compile_and_send.sh`
Copies to `pi@rapspberrypi.local:~/`. Raspberry must be connected to the same network as computer.

## Usage:
`./compile_and_send.sh BINARY_NAME [--release]`

- `--release` flag MUST be after `BINARY_NAME`
- `BINARY_NAME` is mandatory

## Examples:
 `./compile_and_send.sh lidarino_cli --release`
 `./compile_and_send.sh http_server `
