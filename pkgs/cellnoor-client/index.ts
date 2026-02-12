import createClient, { type Client, type ClientOptions } from "openapi-fetch";
import type { paths } from "./api";
import { gzipSync } from "zlib"; // Node.js/Bun only - compression is server-side
export type * from "./api.d.ts";

export type CellnoorClient = Client<paths>;

export interface CellnoorClientOptions extends ClientOptions {
  /**
   * Enable compression of request bodies using gzip.
   * Default: false
   * 
   * Note: This feature requires Node.js or Bun runtime environment
   * and will only work in server-side contexts (e.g., SvelteKit server routes).
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
        let bodyToCompress: string | ArrayBuffer | null = null;
        
        // Handle different body types
        if (typeof init.body === "string") {
          bodyToCompress = init.body;
        } else if (init.body instanceof ArrayBuffer) {
          bodyToCompress = init.body;
        } else if (init.body instanceof Blob) {
          bodyToCompress = await init.body.text();
        } else if (typeof init.body === "object") {
          // Assume it's JSON
          bodyToCompress = JSON.stringify(init.body);
        }
        
        if (bodyToCompress) {
          // Compress the body using gzip
          const compressed = gzipSync(Buffer.from(bodyToCompress));
          
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
  
  // If compression is enabled, wrap the fetch function
  if (options?.compressRequests) {
    const fetchToWrap = options.fetch || fetch;
    clientOptions.fetch = createCompressingFetch(fetchToWrap);
  }
  
  return createClient<paths>(clientOptions);
}
