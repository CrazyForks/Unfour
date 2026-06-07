import { defineConfig } from "vitest/config";
import { resolve } from "node:path";

export default defineConfig({
  resolve: {
    alias: {
      "@unfour/command-client": resolve(__dirname, "packages/command-client/src"),
      "@unfour/workspace": resolve(__dirname, "packages/workspace/src"),
      "@unfour/ui": resolve(__dirname, "packages/ui/src"),
    },
  },
  test: {
    include: ["packages/*/src/**/*.test.ts", "apps/*/src/**/*.test.ts"],
    environment: "node",
  },
});
