/**
 * Formats a byte count into a human-readable string (B, KB, MB, GB, TB)
 */
export function formatSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(2)} ${units[i]}`;
}

/**
 * Formats speed in bytes per second
 */
export function formatSpeed(bytesPerSec: number): string {
  return `${formatSize(bytesPerSec)}/s`;
}

/**
 * Formats ETA in human-readable time (h, m, s)
 */
export function formatEta(seconds: number): string {
  if (seconds <= 0 || !isFinite(seconds)) return "--";
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  if (h > 0) return `${h}h ${m}m ${s}s`;
  if (m > 0) return `${m}m ${s}s`;
  return `${s}s`;
}

/**
 * Calculates percentage safely
 */
export function calcPercent(processed: number, total: number): string {
  if (total <= 0) return "0.0";
  return Math.min((processed / total) * 100, 100).toFixed(1);
}
