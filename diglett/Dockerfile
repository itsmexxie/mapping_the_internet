# https://kerkour.com/rust-small-docker-image#from-alpine
# https://www.codefeetime.com/post/docker-config-for-actix-web-diesel-and-postgres/

# Must be ran from ../

#############
## Builder ##
#############
FROM rust:latest AS builder

WORKDIR /diglett
COPY ./diglett/src ./src
COPY ./diglett/Cargo.toml .
COPY ./diglett/Cargo.lock .
COPY ./lib /lib

RUN cargo build --release

#################
## Final image ##
#################
FROM gcr.io/distroless/cc

# Copy our build
COPY --from=builder /diglett/target/release/diglett .

CMD ["/diglett"]
