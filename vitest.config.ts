import { defineConfig } from "vitest/config";
import { resolve } from "node:path";

export default defineConfig({
  resolve: {
    alias: {
      "@unfour/command-client": resolve(__dirname, "packages/command-client/src"),
      "@unfour/workspace-core": resolve(__dirname, "packages/workspace-core/src"),
      "@unfour/workspace-local": resolve(__dirname, "packages/workspace-local/src"),
      "@unfour/ui": resolve(__dirname, "packages/ui/src"),
    },
  },
  test: {
    include: ["packages/*/src/**/*.test.ts", "apps/*/src/**/*.test.ts"],
    environment: "node",
  },
});
