# syntax=docker/dockerfile:1

ARG RUST_VERSION=1
ARG BUILD_PROFILE=release

FROM rust:${RUST_VERSION}-trixie AS build
ARG BUILD_PROFILE
WORKDIR /app

# Build the app, bind-mounting all the workspace crates and cache-mounting useful directories
RUN --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=cellnoor-api,target=cellnoor-api \
    --mount=type=bind,source=crates,target=crates \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    CELLNOOR_GENERATE_SCHEMA=false cargo build --profile ${BUILD_PROFILE} --package cellnoor-api && \
    if [ ${BUILD_PROFILE} = "dev" ]; then \
    cp ./target/debug/cellnoor-api /bin/cellnoor-api; \
    else \
    cp ./target/${BUILD_PROFILE}/cellnoor-api /bin/cellnoor-api; \
    fi

FROM debian:trixie AS final

RUN apt update && \
    apt install adduser ca-certificates curl zstd postgresql-common --yes

# cellnoor requires `pg_dumpall`, which is in the `postgresql-client-18` package. It takes a little special setup to
# get it (https://www.postgresql.org/download/linux/debian/)
RUN /usr/share/postgresql-common/pgdg/apt.postgresql.org.sh -y && \
    apt update && \
    apt install postgresql-client-18 --yes


RUN mkdir app

COPY --from=build /bin/cellnoor-api /bin/

ARG UID=10001
ARG ADDRESS
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
RUN mkdir -p logs "$(dirname ${ADDRESS})" && chown appuser logs "$(dirname ${ADDRESS})"
USER appuser

CMD /bin/cellnoor-api --mode ${MODE:-production} --db-host ${DB_HOST} --db-port ${DB_PORT} --jwt-issuer ${JWT_ISSUER} --jwt-audience ${JWT_AUDIENCE} --address ${ADDRESS} --db-root-user ${DB_ROOT_USER} --config-dir /run/secrets
