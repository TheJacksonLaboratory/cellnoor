import adapter from "@sveltejs/adapter-node";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";
import { Config } from "@sveltejs/kit";

const config: Config = {
  compilerOptions: {
    experimental: {
      async: true,
    },
  },
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter(),
    csp: {
      directives: {
        "base-uri": ["self"],
        "default-src": ["self"],
        "img-src": ["self", "data:"],
        "style-src": [
          "self",
          "unsafe-inline",
        ],
        "frame-ancestors": ["none"],
        "form-action": ["self"],
      },
      mode: "auto",
    },
  },
};

export default config;
