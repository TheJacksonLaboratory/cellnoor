# syntax=docker/dockerfile:1

ARG RUST_VERSION=1

FROM rust:${RUST_VERSION:-1}-bookworm AS build
WORKDIR /app

# Build the app, bind-mounting all the workspace crates and cache-mounting useful directories
RUN --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=default-vec,target=default-vec \
    --mount=type=bind,source=non-empty,target=non-empty \
    --mount=type=bind,source=scamplers-schema,target=scamplers-schema \
    --mount=type=bind,source=scamplers-models,target=scamplers-models \
    --mount=type=bind,source=scamplers-jsonschema,target=scamplers-jsonschema \
    --mount=type=bind,source=scamplers-api,target=scamplers-api \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    RUN_DIESEL_CLI=false cargo build --release --package scamplers-api && \
    cp ./target/release/scamplers-api /bin/scamplers-api

FROM debian:bookworm AS final

RUN apt update && apt install curl libpq5 --yes

RUN mkdir app

COPY --from=build /bin/scamplers-api /bin/

ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
RUN mkdir logs && chown appuser logs
USER appuser

EXPOSE ${PORT:-80}

CMD /bin/scamplers-api --mode ${MODE:-production} --db-host ${DB_HOST} --db-port ${DB_PORT} --api-key-prefix-length ${API_KEY_PREFIX_LENGTH} --host 0.0.0.0 --port ${PORT} --db-root-user ${DB_ROOT_USER} --log-dir logs --config-dir /run/secrets
