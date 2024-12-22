// @ts-check
import { defineConfig } from "astro/config";
import tailwind from "@astrojs/tailwind";
import sitemap from "@astrojs/sitemap";
import preact from "@astrojs/preact";

// https://astro.build/config
export default defineConfig({
  site: process.env.DEV
    ? "https://localhost:4321"
    : `https://${process.env.RAILWAY_PUBLIC_DOMAIN}`,
  integrations: [
    tailwind(),
    sitemap({
      changefreq: "monthly",
      priority: 1.0,
      // xslURL: "/sitemap.xsl",
    }),
    preact({
      devtools: process.env.DEV != undefined ? true : false,
    }),
  ],
});
