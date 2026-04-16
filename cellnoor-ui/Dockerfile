FROM oven/bun:1 AS base
WORKDIR /usr/src/app

FROM base AS install
COPY package.json bun.lock ./
COPY cellnoor-ui/package.json cellnoor-ui/package.json
COPY pkgs/cellnoor-client/package.json pkgs/cellnoor-client/package.json
RUN bun install --frozen-lockfile

FROM base AS prerelease
COPY --from=install /usr/src/app/node_modules node_modules
COPY --from=install /usr/src/app/cellnoor-ui/node_modules cellnoor-ui/node_modules
COPY --from=install /usr/src/app/pkgs/cellnoor-client/node_modules pkgs/cellnoor-client/node_modules
COPY package.json bun.lock ./
COPY cellnoor-ui/ cellnoor-ui/
COPY pkgs/cellnoor-client/ pkgs/cellnoor-client/
RUN bun run --bun --filter cellnoor-ui build

FROM base AS final

ARG UID
ARG SOCKET_PATH

COPY --from=install /usr/src/app/node_modules node_modules
COPY --from=install /usr/src/app/cellnoor-ui/node_modules cellnoor-ui/node_modules
COPY --from=install /usr/src/app/pkgs/cellnoor-client/node_modules pkgs/cellnoor-client/node_modules
COPY --from=prerelease /usr/src/app/package.json /usr/src/app/bun.lock ./
COPY --from=prerelease /usr/src/app/cellnoor-ui/build/ cellnoor-ui/build/
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

# Run the app. Make sure to delete the socket before every restart cus it's a hassle to do it from within JS
USER appuser

CMD rm -f ${SOCKET_PATH} &&  bun run cellnoor-ui/build/index.js
