# Mapping the Internet - Pokedex
The service list and authentication provider

## Usage
### Cargo
1. Install [rust](https://www.rust-lang.org/learn/get-started)
2. Create a `config.toml` file akin to [this](./config/config.toml) template
3. Run with `cargo run`

### Docker
Build with `docker build -f pidgeotto/Dockerfile .` (must be ran from the root of this repo!). Than run the resulting image with the following command:
```
docker run --rm -v ./jwt.key.pub:/jwt.key.pub -v ./jwt.key:/jwt.key -v ./config.toml:/config.toml -p {{ HOST_PORT }}:{{ API_PORT }} {{ IMAGE_ID }}
```

The API_PORT in the docker run command must be the same one as the one you specified in the config.

## Database
The unit needs access to the following tables with the following permissions:
- ServiceUnits (*)
- Services (*)
