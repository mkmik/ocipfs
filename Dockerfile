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
FROM gcr.io/distroless/cc@sha256:aa86ff4d07167d636f2dad015468d64a37bdcb4b7874a9582d3700fac9d9f542
COPY --from=builder /out/ /bin/
ENV ROCKET_PORT=8080
ENTRYPOINT [ "/bin/ocipfs" ]



