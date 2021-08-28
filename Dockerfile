# Dockerfile for creating a statically-linked Rust application using docker's
# multi-stage build feature. This also leverages the docker build cache to avoid
# re-downloading dependencies if they have not changed.
FROM rust:latest AS cargo-build
WORKDIR /usr/src/rust-axum-scylla

# Download the target for static linking.
#RUN rustup target add x86_64-unknown-linux-musl

# Create a dummy project and build the app's dependencies.
# If the Cargo.toml or Cargo.lock files have not changed,
# we can use the docker build cache and skip these (typically slow) steps.
COPY Cargo.toml Cargo.lock ./

# Copy the source and build the application.
COPY src ./src
RUN cargo build --release
#RUN cargo install --target x86_64-unknown-linux-musl --path .
RUN cargo install --path .

# Copy the statically-linked binary into a scratch container.
FROM scratch
COPY --from=cargo-build /usr/local/cargo/bin/hello /usr/local/bin/hello
ENTRYPOINT ["hello"]
