# syntax=docker/dockerfile:1

FROM oven/bun:1 AS build

WORKDIR /usr/src/app
COPY . .
RUN bun install --frozen-lockfile --filter cellnoor-ui

WORKDIR /usr/src/app/cellnoor-ui
RUN bun run build

FROM debian:trixie AS final

ARG UI_DIR
ENV UI_DIR=${UI_DIR}

COPY --from=build /usr/src/app/cellnoor-ui/build /cellnoor-ui

CMD ["/bin/sh", "-c", "find \"$UI_DIR\" -mindepth 1 -delete && cp -r /cellnoor-ui/. \"$UI_DIR\""]
