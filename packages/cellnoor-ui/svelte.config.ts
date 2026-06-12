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
      // TODO: Make this more permissive
      directives: {
        "base-uri": ["self"],
        "default-src": ["self"],
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
