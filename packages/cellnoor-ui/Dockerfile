FROM oven/bun:1 AS build

WORKDIR /usr/src/app

COPY * .
RUN bun install --frozen-lockfile && bun run build

FROM debian:trixie AS final

COPY --from=build /usr/src/app/build /bin/cellnoor-auth
