import { resolve } from "path";
import { defineConfig } from "vite";

export default defineConfig({
    server: {
        port: 5001,
    },
    build: {
        outDir: "./public",
        assetsDir: "./assets",
        target: "esnext",
        rollupOptions: {
            input: {
                "index": resolve(__dirname, "index.html"),
                "custom-app": resolve(__dirname, "custom-app.html"),
            },
          },
    },
});
