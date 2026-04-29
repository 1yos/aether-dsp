import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Windows junction workaround:
// C:\aether-dsp is a junction to D:\Audio kernel\aether-dsp (has spaces).
// Rollup resolves the junction to the real path, which breaks because of spaces.
// Fix: explicitly set the rollupOptions.input to use the CWD-relative path,
// which stays as the junction path when npm is run from C:\aether-dsp\ui.
export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
  },
  build: {
    rollupOptions: {
      input: "index.html",
    },
  },
});
