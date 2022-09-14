# syntax = docker/dockerfile:1-experimental

# Build Stage
FROM rust:1.63.0@sha256:54f17dc625ed83d3fc7dee68d5eac4e1977de40e72d909bcdb50cc55a462779d AS builder

ENV CARGO_TARGET_DIR=/tmp/target
ENV CARGO_HOME=/tmp/cargo

ENV BIN=ocipfs

WORKDIR /usr/src/
RUN mkdir /out

RUN USER=root cargo new builder
WORKDIR /usr/src/builder

COPY rust-toolchain.toml .

RUN --mount=type=cache,target=/tmp/cargo \
    --mount=type=cache,target=/tmp/target \
    cargo build --release

COPY . .

RUN --mount=type=cache,target=/tmp/cargo \
    --mount=type=cache,target=/tmp/target \
    cargo build --release && \
    cp ${CARGO_TARGET_DIR}/release/${BIN} /out/

# Bundle Stage
FROM gcr.io/distroless/cc@sha256:15bc3c6acbe16bf3dc37db543dfa632dabf3869ccd123257e9bee965989c976e
COPY --from=builder /out/ /bin/
ENV ROCKET_PORT=8080
ENTRYPOINT [ "/bin/ocipfs" ]



