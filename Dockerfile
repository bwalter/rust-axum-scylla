# stage 1: build

FROM rust:1.55.0-alpine3.14 as builder
RUN apk add build-base
RUN apk add openssl-dev

WORKDIR /home/rust

# First dummy project to cache all dependencies
COPY ./Cargo.lock ./Cargo.toml ./
RUN mkdir src
RUN echo "fn main() {}" > src/main.rs
RUN cargo test
RUN cargo build --release

# Copy real files and touch main.rs or otherwise dock will use the cached one
COPY . .
RUN touch src/main.rs

# Test, build and strip real project
RUN cargo test --lib
RUN cargo build --release
RUN strip target/release/hello

# stage 2: run

FROM alpine:3.14

COPY --from=builder /home/rust/target/release/hello .

EXPOSE 3000
ENV RUST_LOG=hello=debug,tower_http::trace=debug
ENTRYPOINT ["./hello"]

