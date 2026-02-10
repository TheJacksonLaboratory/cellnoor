FROM oven/bun:1 AS final
WORKDIR /usr/src/app

# Copy the necessary files into the image (this is going to result in a bloated image, but I don't care because I hate
# building JavaScript projects
COPY package.json bun.lock ./
COPY cellnoor-ui/ ./cellnoor-ui/
COPY pkgs/cellnoor-client/ ./pkgs/cellnoor-client/
RUN bun install --frozen-lockfile
RUN bun run --bun --filter cellnoor-ui build
RUN apt update && apt install curl --yes

# run the app
USER bun
EXPOSE ${PORT:-80}

CMD bun run cellnoor-ui/build/index.js
