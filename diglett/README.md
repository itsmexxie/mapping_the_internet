# Mapping the Internet - Diglett
A service that provides a nice HTTP abstraction over services that provide address information via text files.

## Service list
The services which we are currently using to get information are as follows:
- ftp.arin.net (allocation states, RIRs, countries)
- IANA number resources (RIRs, reserved blocks)
- thyme.apnic.net (RIRs, ASNs)

## Usage
### Cargo
1. Install [rust](https://www.rust-lang.org/learn/get-started)
2. Create a `config.toml` file akin to [this](./config/config.toml) template
3. Run with `cargo run`

### Docker
Build with `docker build -f diglett/Dockerfile .` (must be ran from the root of this repo!). Then run the resulting image with the following command:
```
docker run --rm -v ./jwt.key.pub:/jwt.key.pub -v ./config.toml:/config.toml -p 80:{{ API_PORT }} {{ IMAGE_ID }}
```

The API_PORT in the docker run command must be the same one as the one you specified in the config.
