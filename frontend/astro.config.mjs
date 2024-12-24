// @ts-check
import { defineConfig } from "astro/config";
import tailwind from "@astrojs/tailwind";
import sitemap from "@astrojs/sitemap";
import preact from "@astrojs/preact";

// TODO: Add linting to build steps

// https://astro.build/config
export default defineConfig({
  build: {
    assets: "assets",
  },
  site: import.meta.env.DEV
    ? "https://localhost:4321"
    : `https://${import.meta.env.RAILWAY_PUBLIC_DOMAIN}`,
  integrations: [
    tailwind(),
    sitemap({
      changefreq: "monthly",
      priority: 1.0,
      // xslURL: "/sitemap.xsl",
    }),
    preact({
      devtools: import.meta.env.DEV ?? false,
    }),
  ],
});
