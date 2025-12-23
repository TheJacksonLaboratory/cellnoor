# Adapted from https://bun.com/docs/guides/ecosystem/docker. This could probably be improved to be more consistent with
# cellnoor-api.Dockerfile but for now this is fine
FROM oven/bun:1 AS base
WORKDIR /usr/src/app

FROM base AS install
RUN mkdir -p /temp/dev
COPY cellnoor-ui/package.json cellnoor-ui/bun.lock /temp/dev/
RUN cd /temp/dev && bun install --frozen-lockfile

RUN mkdir -p /temp/prod
COPY cellnoor-ui/package.json cellnoor-ui/bun.lock /temp/prod/
RUN cd /temp/prod && bun install --frozen-lockfile --production

FROM base AS prerelease
COPY --from=install /temp/dev/node_modules node_modules
COPY cellnoor-ui/ .

RUN bun run --bun build

FROM base AS final
RUN apt update && apt install curl --yes

COPY --from=install /temp/prod/node_modules node_modules
COPY --from=prerelease /usr/src/app/build .
COPY --from=prerelease /usr/src/app/package.json .

# run the app
USER bun
EXPOSE ${PORT:-80}

CMD IN_DOCKER=true bun run index.js
