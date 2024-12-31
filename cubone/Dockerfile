# https://kerkour.com/rust-small-docker-image#from-alpine
# https://www.codefeetime.com/post/docker-config-for-actix-web-diesel-and-postgres/

# Must be ran from ../

#############
## Builder ##
#############
FROM rust:latest AS builder

# Build cubone
WORKDIR /cubone
COPY ./cubone/src ./src
COPY ./cubone/Cargo.toml .
COPY ./cubone/Cargo.lock .
COPY ./lib /lib

RUN cargo build --release

# Build healthcheck
WORKDIR /healthcheck
COPY ./healthcheck/src ./src
COPY ./healthcheck/Cargo.toml .
COPY ./healthcheck/Cargo.lock .

RUN cargo build --release

#################
## Final image ##
#################
FROM gcr.io/distroless/cc

# Copy our build
COPY --from=builder /cubone/target/release/cubone .
COPY --from=builder /healthcheck/target/release/healthcheck .
COPY ./cubone/sprite.txt .

CMD ["/cubone"]
