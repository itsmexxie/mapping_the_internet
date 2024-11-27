# https://kerkour.com/rust-small-docker-image#from-alpine
# https://www.codefeetime.com/post/docker-config-for-actix-web-diesel-and-postgres/
#############
## Builder ##
#############
FROM rust:latest AS builder

RUN apt update

WORKDIR /diglett
COPY ./ .

RUN cargo build --release

#################
## Final image ##
#################
FROM gcr.io/distroless/cc

# Copy our build
COPY --from=builder /diglett/target/release/diglett .

CMD ["/diglett"]