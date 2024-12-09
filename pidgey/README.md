# Mapping the Internet - Pidgey
A service which queries data about an IP address using multiple sources

## Usage
### Cargo
1. Install [rust](https://www.rust-lang.org/learn/get-started)
2. Create a `config.toml` file akin to [this](./config/config.toml) template
3. Run with `cargo run`

### Docker
Build with `docker build -f pidgey/Dockerfile .` (must be ran from the root of this repo!). Than run the resulting image with the following command:
```
docker run --rm -v ./jwt.key.pub:/jwt.key.pub -v ./config.toml:/config.toml -p 7020:7020 {{ IMAGE_ID }}
```

The API_PORT in the docker run command must be the same one as the one you specified in the config.

## TODO
- [ ] Pidgeotto service discovery via pokedex
- [x] API auth
