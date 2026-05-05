/**
 * Crash reporter — captures unhandled errors and sends them to a
 * configurable endpoint.
 *
 * Activated by setting VITE_SENTRY_DSN in the environment.
 * No-op if the DSN is not set — zero overhead in development.
 */

interface CrashReport {
  message: string;
  stack?: string;
  url: string;
  userAgent: string;
  timestamp: string;
  appVersion: string;
  context?: Record<string, unknown>;
}

function getDsn(): string | undefined {
  // import.meta.env is typed by vite/client (added to tsconfig types)
  return import.meta.env.VITE_SENTRY_DSN as string | undefined;
}

function getVersion(): string {
  return (import.meta.env.VITE_APP_VERSION as string | undefined) ?? "0.2.0";
}

function sendReport(report: CrashReport) {
  const dsn = getDsn();
  if (!dsn) return;
  const body = JSON.stringify(report);
  if (navigator.sendBeacon) {
    navigator.sendBeacon(dsn, body);
  } else {
    fetch(dsn, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body,
      keepalive: true,
    }).catch(() => {
      /* ignore network errors in crash reporter */
    });
  }
}

function buildReport(
  error: Error | string,
  context?: Record<string, unknown>,
): CrashReport {
  const err = typeof error === "string" ? new Error(error) : error;
  return {
    message: err.message,
    stack: err.stack,
    url: window.location.href,
    userAgent: navigator.userAgent,
    timestamp: new Date().toISOString(),
    appVersion: getVersion(),
    context,
  };
}

/** Initialize global error handlers. Call once at app startup. */
export function initCrashReporter() {
  if (!getDsn()) return;

  window.addEventListener("error", (e) => {
    sendReport(
      buildReport(e.error instanceof Error ? e.error : new Error(e.message), {
        filename: e.filename,
        lineno: e.lineno,
        colno: e.colno,
      }),
    );
  });

  window.addEventListener("unhandledrejection", (e) => {
    const err =
      e.reason instanceof Error ? e.reason : new Error(String(e.reason));
    sendReport(buildReport(err, { type: "unhandledrejection" }));
  });

  console.info("[CrashReporter] initialized");
}

/** Manually report an error with optional context. */
export function reportError(
  error: Error | string,
  context?: Record<string, unknown>,
) {
  sendReport(buildReport(error, context));
}
