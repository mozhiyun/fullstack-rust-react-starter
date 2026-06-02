import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const rootDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(rootDir, "../..");

/** 与 web 的 React 19 隔离，admin 固定走仓库根目录的 React 18 */
const reactRoot = path.resolve(repoRoot, "node_modules/react");
const reactDomRoot = path.resolve(repoRoot, "node_modules/react-dom");

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      react: reactRoot,
      "react-dom": reactDomRoot,
    },
    dedupe: ["react", "react-dom", "react-router"],
  },
  server: {
    port: 5174,
    strictPort: false,
  },
});
