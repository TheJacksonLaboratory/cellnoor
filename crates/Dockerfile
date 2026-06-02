# syntax=docker/dockerfile:1

ARG RUST_VERSION=1
ARG BUILD_PROFILE=release

FROM rust:${RUST_VERSION}-trixie AS build
ARG BUILD_PROFILE
WORKDIR /app

# Build the app, bind-mounting all the workspace crates and cache-mounting useful directories
RUN --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=cellnoor,target=cellnoor \
    --mount=type=bind,source=cellnoor-types,target=cellnoor-types \
    --mount=type=bind,source=nonempty,target=nonempty \
    --mount=type=bind,source=positive,target=positive \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --profile ${BUILD_PROFILE} --package cellnoor && \
    if [ ${BUILD_PROFILE} = "dev" ]; then \
        cp ./target/debug/cellnoor /bin/cellnoor; \
    else \
        cp ./target/${BUILD_PROFILE}/cellnoor /bin/cellnoor; \
    fi

FROM debian:trixie AS final

# TODO: remove zstd if we end up using a pure Rust version
RUN apt update && \
    apt install adduser ca-certificates curl zstd --yes

RUN mkdir app

COPY --from=build /bin/cellnoor /bin/

ARG UID=10001
ARG SOCKET
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
RUN mkdir -p "$(dirname ${SOCKET})" && chown appuser "$(dirname ${SOCKET})"
USER appuser

CMD ["/bin/cellnoor"]
