# cellnoor-client

TypeScript client for the cellnoor API, with built-in support for request compression.

## Features

- **Request Compression**: Automatically compress request bodies using gzip to reduce bandwidth usage
- **TypeScript**: Fully typed API client based on OpenAPI specification
- **Flexible**: Works with custom fetch implementations

## Installation

```bash
bun install
```

## Usage

### Basic Usage

```typescript
import { createCellnoorClient } from "cellnoor-client";

const client = createCellnoorClient({
  baseUrl: "https://api.example.com",
});
```

### With Request Compression

Enable request compression by setting `compressRequests: true`:

```typescript
import { createCellnoorClient } from "cellnoor-client";

const client = createCellnoorClient({
  baseUrl: "https://api.example.com",
  compressRequests: true, // Enable gzip compression for request bodies
  fetch: customFetch, // Optional: provide custom fetch implementation
});
```

When enabled, the client will:
1. Compress request bodies using gzip
2. Add the `Content-Encoding: gzip` header
3. Fall back to uncompressed requests if compression fails

**Note**: The server must support decompressing request bodies for this feature to work properly.

## Development

This project was created using `bun init` in bun v1.3.5. [Bun](https://bun.com)
is a fast all-in-one JavaScript runtime.
