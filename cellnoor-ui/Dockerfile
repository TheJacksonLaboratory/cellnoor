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
COPY --from=install /usr/src/app/node_modules node_modules
COPY --from=install /usr/src/app/cellnoor-ui/node_modules cellnoor-ui/node_modules
COPY --from=install /usr/src/app/pkgs/cellnoor-client/node_modules pkgs/cellnoor-client/node_modules
COPY --from=prerelease /usr/src/app/package.json /usr/src/app/bun.lock ./
COPY --from=prerelease /usr/src/app/cellnoor-ui/build/ cellnoor-ui/build/
RUN apt update && apt install curl --yes

# run the app
USER bun

CMD bun run cellnoor-ui/build/index.js
