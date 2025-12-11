// @ts-check
import { defineConfig } from "astro/config";
import tailwind from "@astrojs/tailwind";
import sitemap from "@astrojs/sitemap";

import react from "@astrojs/react";

// Log warning if RAILWAY_PUBLIC_DOMAIN is not set in production
if (
  process.env.NODE_ENV === "production" &&
  !process.env.RAILWAY_PUBLIC_DOMAIN
) {
  console.warn(
    "⚠️  RAILWAY_PUBLIC_DOMAIN not set in production mode. Falling back to https://dynamic-preauth.xevion.dev"
  );
}

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
          process.env.RAILWAY_PUBLIC_DOMAIN || "dynamic-preauth.xevion.dev"
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
