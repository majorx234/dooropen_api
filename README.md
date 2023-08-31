# Generation OpenApi Backend:
* `openapi-spec-validator --schema 3.0.0 dooropen.yaml`
* `openapi-generator-cli generate -i dooropen.yaml -g rust-server --additional-properties=packageName=dooropen_api`

# build
* `cargo build`

## build for other environments (e.g. aarch64)
### preparation
-   use `rustup` (as system package in Archlinux) because of the different toolchains you can easily download with it
-   download the target [toolchain](https://doc.rust-lang.org/nightly/rustc/platform-support.html)] and run:
    -   `rustup target add --toolchain stable-glibc aarch64-unknown-linux-gnu`
-   then you need an appropriate linker for the target system, e.g. in arch exist a package for aarch64: 
    -   `sudo pacman -S aarch64-linux-gnu-gcc`
-   and finally you need to specifiy the linker for the toolchain in the cargo config (e.g.`$HOME/.cargo/config`):
```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
```
-   NOTE: check the version of the installed linker and the one in the target system

### build command
* `cargo build --target=aarch64-unknown-linux-gnu` 
Or add it in the config

# usage
## start server
* `cargo run --package dooropen`
## test with curl
* `curl --request GET http://127.0.0.1:8080/v1.0/ping -v`
