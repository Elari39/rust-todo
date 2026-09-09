/// <reference types="vitest/config" />
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import process from "node:process";
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(() => ({
  plugins: [vue()],

  // vitest：纯逻辑测试跑 node 环境即可（组件测试将来需要换成 jsdom/happy-dom）
  test: {
    environment: "node",
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri` and local session state
      //    (.mimosa files get locked by the session process and EBUSY-crash the watcher)
      ignored: ["**/src-tauri/**", "**/.mimosa/**", "**/.zcode/**", "**/node_modules/**"],
    },
  },
}));
