# Mapping the Internet - Cubone
A service which provides an HTTP API for querying the database about address data.

## Usage
### Cargo
1. Install [rust](https://www.rust-lang.org/learn/get-started)
2. Create a `config.toml` file akin to [this](./config/config.toml) template
3. Run with `cargo run`

### Docker
Build with `docker build -f cubone/Dockerfile .` (must be ran from the root of this repo!). Then run the resulting image with the following command:
```
docker run --rm -v ./config.toml:/config.toml -p 80:{{ API_PORT }} {{ IMAGE_ID }}
```

The API_PORT in the docker run command must be the same one as the one you specified in the config.
