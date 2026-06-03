FROM oven/bun:1 AS build

WORKDIR /usr/src/app

COPY * .
RUN bun install --frozen-lockfile && bun build --compile --minify --bytecode --sourcemap --format esm --define Bun.env.READ_SECRET_FILE=true index.ts --outfile cellnoor-auth


FROM debian:trixie AS final

COPY --from=build /usr/src/app/cellnoor-auth /bin/cellnoor-auth

# We make sure to create the socket directory and then remove the socket file if it already exists
RUN mkdir -p /run/cellnoor-auth

# TODO: switch to an unprivileged user
CMD ["/bin/sh", "-c", "rm -f /run/cellnoor-auth/cellnoor-auth.sock && /bin/cellnoor-auth"]
