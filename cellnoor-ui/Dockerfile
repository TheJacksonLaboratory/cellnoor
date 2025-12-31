# Adapted from https://bun.com/docs/guides/ecosystem/docker. It's an absolute mess
FROM oven/bun:1 AS base
WORKDIR /usr/src/app

FROM base AS install
RUN mkdir -p /temp/dev/pkgs/cellnoor-types /temp/dev/cellnoor-ui
COPY package.json bun.lock /temp/dev/
COPY cellnoor-ui/package.json /temp/dev/cellnoor-ui
COPY pkgs/cellnoor-types/package.json /temp/dev/pkgs/cellnoor-types/
RUN cd /temp/dev && bun install --frozen-lockfile

RUN mkdir -p /temp/prod/pkgs/cellnoor-types /temp/prod/cellnoor-ui
COPY package.json bun.lock /temp/prod/
COPY cellnoor-ui/package.json /temp/prod/cellnoor-ui
COPY pkgs/cellnoor-types/package.json /temp/prod/pkgs/cellnoor-types/
RUN cd /temp/prod && bun install --frozen-lockfile --production

FROM base AS prerelease
COPY --from=install /temp/dev/node_modules node_modules
COPY package.json bun.lock ./
RUN mkdir -p cellnoor-ui pkgs/cellnoor-types
COPY cellnoor-ui/ ./cellnoor-ui
COPY pkgs/cellnoor-types/ ./pkgs/cellnoor-types/

RUN bun install && bun run --bun --cwd cellnoor-ui build

FROM base AS final
RUN apt update && apt install curl --yes

COPY --from=install /temp/prod/node_modules node_modules
COPY --from=prerelease /usr/src/app/cellnoor-ui/build .
COPY --from=prerelease /usr/src/app/cellnoor-ui/package.json .

# run the app
USER bun
EXPOSE ${PORT:-80}

CMD bun run index.js
