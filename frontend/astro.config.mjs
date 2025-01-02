// @ts-check
import { defineConfig, envField } from "astro/config";
import tailwind from "@astrojs/tailwind";
import sitemap from "@astrojs/sitemap";

import react from "@astrojs/react";

// TODO: Add linting to build steps

// https://astro.build/config

export default defineConfig({
  build: {
    assets: "assets",
  },
  site:
    process.env.NODE_ENV === "development"
      ? "https://localhost:4321"
      : `https://${
          process.env.RAILWAY_PUBLIC_DOMAIN ??
          (() => {
            throw new Error("RAILWAY_PUBLIC_DOMAIN not set");
          })()
        }`,
  integrations: [
    tailwind(),
    sitemap({
      changefreq: "monthly",
      priority: 1.0,
      // xslURL: "/sitemap.xsl",
    }),
    react(),
  ],
});
