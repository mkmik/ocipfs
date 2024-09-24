# syntax = docker/dockerfile:1-experimental@sha256:600e5c62eedff338b3f7a0850beb7c05866e0ef27b2d2e8c02aa468e78496ff5

# Build Stage
FROM rust:1.65.0@sha256:891bc3b252c43a1c2667083e3861f26e6f571dcc3bc98dcc151d6ff6edc62cb9 AS builder

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
FROM gcr.io/distroless/cc@sha256:3310655aac0d85eb9d579792387af1ff3eb7a1667823478be58020ab0e0d97a8
COPY --from=builder /out/ /bin/
ENV ROCKET_PORT=8080
ENTRYPOINT [ "/bin/ocipfs" ]



