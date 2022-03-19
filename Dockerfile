# syntax = docker/dockerfile:1-experimental

# Build Stage
FROM rust:1.59.0@sha256:fea232803139b909ba783c334b2b634caa10d70f0bf46ed2106233459dd032f4 AS builder

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
FROM gcr.io/distroless/cc@sha256:a9e2593c4ebca5435d8b23ef4316323842fc36b0410bf9a5d3f472ab5b52d0f0
COPY --from=builder /out/ /bin/
ENV ROCKET_PORT=8080
ENTRYPOINT [ "/bin/ocipfs" ]



