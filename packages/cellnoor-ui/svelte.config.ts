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
        "default-src": ["self"],
        // We hardcode everything here because this is a deprecated trash app, but in el futuro it will use a build-time variable
        "frame-src": [
           "self",
           "*.cellnoor.jax.org",
           "cellnoor.jax.org:8001",
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
