import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { initCrashReporter } from "./lib/crashReporter";

// Initialize crash reporting before anything else.
// No-op unless VITE_SENTRY_DSN is set in the environment.
initCrashReporter();

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
