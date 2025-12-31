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
          "unsafe-hashes",
          "sha256-tcbDxjMo+xKqM21aCGYbs/QAJqB7yUXC06oPWDapBgc=",
          "sha256-S8qMpvofolR8Mpjy4kQvEm7m1q8clzU4dfDH0AmvZjo=",
        ],
        "frame-ancestors": ["none"],
        "form-action": ["self"],
      },
      mode: "auto",
    },
  },
};

export default config;
