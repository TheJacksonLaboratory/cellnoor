import adapter from "@sveltejs/adapter-node";
import { type Config } from "@sveltejs/kit";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

const config: Config = {
  compilerOptions: {
    experimental: {
      async: true,
    },
  },
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter(),
    experimental: { explicitEnvironmentVariables: true, remoteFunctions: true },
    csp: {
      directives: {
        "base-uri": ["self"],
        "default-src": [
          "self",
          "*.cellnoor.jax.org:3000",
          "*.cellnoor.jax.org:3001",
          "*.cellnoor.jax.org:3002",
          "*.cellnoor.localhost",
        ],
        "frame-src": [
          "self",
          "*.cellnoor.jax.org:3000",
          "*.cellnoor.jax.org:3001",
          "*.cellnoor.jax.org:3002",
          "*.cellnoor.localhost",
        ],
        "img-src": ["self", "data:"],
        "style-src": ["self", "unsafe-inline"],
        "frame-ancestors": ["none"],
        "form-action": ["self"],
      },
      mode: "auto",
    },
  },
};

export default config;
