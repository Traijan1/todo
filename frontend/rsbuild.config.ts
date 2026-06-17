import { defineConfig } from "@rsbuild/core";
import { pluginVue } from "@rsbuild/plugin-vue";

export default defineConfig({
  plugins: [pluginVue()],
  html: {
    favicon: "src/assets/favicon.ico",
    title: "Todo",
    meta: {
      viewport: "width=device-width, initial-scale=1.0, viewport-fit=cover",
    },
    tags: [
      { tag: "link", attrs: { rel: "manifest", href: "/manifest.webmanifest" } },
      { tag: "meta", attrs: { name: "theme-color", content: "#E0BBE4" } },
      { tag: "meta", attrs: { name: "mobile-web-app-capable", content: "yes" } },
      { tag: "meta", attrs: { name: "apple-mobile-web-app-capable", content: "yes" } },
      { tag: "meta", attrs: { name: "apple-mobile-web-app-status-bar-style", content: "black-translucent" } },
      { tag: "meta", attrs: { name: "apple-mobile-web-app-title", content: "Todo" } },
      { tag: "link", attrs: { rel: "apple-touch-icon", href: "/apple-touch-icon.png" } },
    ],
  },
  server: {
    proxy: {
      "/api": {
        target: "http://localhost:5150",
        changeOrigin: true,
        secure: false,
        ws: true,
        pathRewrite: { "^/api": "/api" },
      },
    },
  },
});
