import createClient, { type Client, type ClientOptions } from "openapi-fetch";
import type { paths } from "./api";
import { gzipSync } from "zlib";
export type * from "./api.d.ts";

export type CellnoorClient = Client<paths>;

export interface CellnoorClientOptions extends ClientOptions {
  /**
   * Enable compression of request bodies using gzip.
   * Default: false
   */
  compressRequests?: boolean;
}

/**
 * Creates a fetch wrapper that compresses request bodies
 */
function createCompressingFetch(originalFetch: typeof fetch): typeof fetch {
  return async (input: RequestInfo | URL, init?: RequestInit) => {
    // If there's a body, compress it
    if (init?.body) {
      try {
        let bodyData: string | ArrayBuffer | null = null;
        
        // Handle different body types
        if (typeof init.body === "string") {
          bodyData = init.body;
        } else if (init.body instanceof ArrayBuffer) {
          bodyData = init.body;
        } else if (init.body instanceof Blob) {
          bodyData = await init.body.text();
        } else if (typeof init.body === "object") {
          // Assume it's JSON
          bodyData = JSON.stringify(init.body);
        }
        
        if (bodyData) {
          // Compress the body using gzip
          const compressed = gzipSync(Buffer.from(bodyData));
          
          // Create new init with compressed body
          const newInit = {
            ...init,
            body: compressed,
            headers: {
              ...init.headers,
              "Content-Encoding": "gzip",
            },
          };
          
          return originalFetch(input, newInit);
        }
      } catch (error) {
        // If compression fails, continue with uncompressed request
        console.warn("Failed to compress request body:", error);
      }
    }
    
    // No body or compression failed, use original fetch
    return originalFetch(input, init);
  };
}

export function createCellnoorClient(options?: CellnoorClientOptions) {
  const clientOptions = { ...options };
  
  // If compression is enabled and a custom fetch is provided, wrap it
  if (options?.compressRequests && options.fetch) {
    clientOptions.fetch = createCompressingFetch(options.fetch);
  }
  
  return createClient<paths>(clientOptions);
}
