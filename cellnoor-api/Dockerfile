# syntax=docker/dockerfile:1

ARG RUST_VERSION=1

FROM rust:${RUST_VERSION:-1}-bookworm AS build
WORKDIR /app

RUN apt update && apt install libpq5 --yes

# Build the app, bind-mounting all the workspace crates and cache-mounting useful directories
RUN --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=cellnoor-api,target=cellnoor-api \
    --mount=type=bind,source=crates,target=crates \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    CELLNOOR_GENERATE_SCHEMA=false cargo build --release --package cellnoor-api && \
    cp ./target/release/cellnoor-api /bin/cellnoor-api

FROM debian:bookworm AS final

RUN apt update && apt install curl libpq5 --yes

RUN mkdir app

COPY --from=build /bin/cellnoor-api /bin/

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

CMD /bin/cellnoor-api --mode ${MODE:-production} --db-host ${DB_HOST} --db-port ${DB_PORT} --api-key-prefix-length ${API_KEY_PREFIX_LENGTH} --host 0.0.0.0 --port ${PORT} --db-root-user ${DB_ROOT_USER} --log-dir logs --config-dir /run/secrets
