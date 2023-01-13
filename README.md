# Generation OpenApi Backend:
* `openapi-spec-validator --schema 3.0.0 dooropen.yaml`
* `openapi-generator-cli generate -i dooropen.yaml -g rust-server --additional-properties=packageName=dooropen_api`

# build
* `cargo build`

# usage
## start server
* `cargo run --package dooropen`
## test with curl
* `curl --request GET http://127.0.0.1:8080/v1.0/ping -v`
