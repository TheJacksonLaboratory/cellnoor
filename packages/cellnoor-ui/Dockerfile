FROM oven/bun:1 AS build
WORKDIR /usr/src/app

COPY package.json bun.lock ./
RUN bun install --frozen-lockfile --dev
COPY . .
ENV NODE_ENV=production
RUN bun run --bun build

FROM oven/bun:1 AS final
WORKDIR /usr/src/app

ARG UID
ARG SOCKET_PATH

# curl is used by the compose healthcheck
RUN apt update && apt install curl adduser --yes

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser

RUN mkdir -p "$(dirname ${SOCKET_PATH})" && chown appuser "$(dirname ${SOCKET_PATH})"

COPY --from=build /usr/src/app/package.json ./
COPY --from=build /usr/src/app/node_modules node_modules
COPY --from=build /usr/src/app/build build

# Run the app. Make sure to delete the socket before every restart cus it's a hassle to do it from within JS
USER appuser

ENV NODE_ENV=production
CMD ["/bin/sh", "-c", "rm -f /run/cellnoor-ui/cellnoor-ui.sock && bun run build/index.js"]
