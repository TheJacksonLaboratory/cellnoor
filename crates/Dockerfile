# syntax=docker/dockerfile:1

ARG RUST_VERSION=1
ARG BUILD_PROFILE=release

FROM rust:${RUST_VERSION}-trixie AS build
ARG BUILD_PROFILE
ARG FEATURES
WORKDIR /app

# Build the app, bind-mounting all the workspace crates and cache-mounting useful directories
RUN --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=cellnoor-api,target=cellnoor-api \
    --mount=type=bind,source=cellnoor-types,target=cellnoor-types \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo install --locked --no-track --path cellnoor-api --bin cellnoor-api --profile ${BUILD_PROFILE} --features "${FEATURES}" --root /

FROM debian:trixie AS final

RUN apt update && \
    apt install ca-certificates curl zstd --yes

COPY --from=build /bin/cellnoor-api /bin/

ARG UID=10001
ARG SOCKET
ARG STATIC_FILES_DIR

RUN mkdir -p "$(dirname ${SOCKET})" ${STATIC_FILES_DIR} && chown ${UID} "$(dirname ${SOCKET})" ${STATIC_FILES_DIR}
USER ${UID}

CMD ["/bin/cellnoor-api"]
