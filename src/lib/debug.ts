export function debugError(...args: unknown[]) {
  if (import.meta.env.DEV) {
    console.error(...args);
  }
}
