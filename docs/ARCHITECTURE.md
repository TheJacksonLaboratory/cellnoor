# **Architecture**

cellnoor utilizes PostgresSQL as its underlying database. The application is essentially 3 services built on top of the database:

- A RESTful API ([crates/cellnoor](./crates/cellnoor)), written in Rust using [axum](https://github.com/tokio-rs/axum) and [tokio-postgres](https://github.com/rust-postgres/rust-postgres). From this RESTful API, we generate [an OpenAPI specification](./openapi.json).
- A single-page application that queries the RESTful API ([packages/cellnoor-ui](./cellnoor-ui)), built with [SolidJS](https://v2.solidjs.com). For end-to-end type-safety, we generate TypeScript type-stubs using [openapi-ts](https://openapi-ts.dev/). These type-stubs are at [cellnoor-types.ts](./packages/cellnoor-ui/src/lib/cellnoor-types.ts).
- A simple authentication service ([packages/cellnoor-auth](./packages/cellnoor-auth)) to easily authenticate users with OpenID-Connect, built with [better-auth](https://better-auth.com).

[Caddy](caddyserver.com) sits as a reverse proxy in front of these 3 services. Its configuration is at [caddy/Caddyfile](./caddy/Caddyfile).
