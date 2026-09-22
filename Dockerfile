# syntax=docker/dockerfile:1.27

ARG RUST_VERSION

FROM rust:${RUST_VERSION?}-slim-trixie AS builder
ARG BIN=tactica-api

# TARGETARCH is supplied by buildx per platform.
ARG TARGETARCH
WORKDIR /src

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry,id=cargo-registry-${TARGETARCH}},sharing=locked \
    --mount=type=cache,target=/src/target,id=cargo-target-${TARGETARCH}},sharing=locked \
    cargo build --release --locked --bin "${BIN}" && \
    cp "target/release/${BIN}" "/usr/local/bin/${BIN}"

FROM debian:trixie-slim AS runtime
ARG BIN=tactica-api

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl tini \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 tactica \
    && useradd --uid 10001 --gid tactica --no-create-home --shell /usr/sbin/nologin tactica

COPY --from=builder --chown=10001:10001 /usr/local/bin/${BIN} /usr/local/bin/${BIN}

LABEL org.opencontainers.image.source="https://git.hayden.moe/tactica/tactica"
USER 10001:10001
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD ["curl", "--max-time", "2", "--fail", "--silent", "http://127.0.0.1:8080/healthz"]

ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/${BIN}"]
