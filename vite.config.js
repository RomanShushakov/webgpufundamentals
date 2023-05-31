import { defineConfig } from "vite";

export default defineConfig({
    server: {
        port: 5001,
    },
    build: {
        outDir: "./public",
        assetsDir: "./assets",
        target: "esnext",
    },
});
