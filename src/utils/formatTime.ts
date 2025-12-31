export function formatTime(hours: number | undefined) {
  const numericHours = Number.isFinite(hours ?? NaN)
    ? (hours as number)
    : Number(hours);
  const usableHours = Math.max(0, numericHours || 0);
  const totalMinutes = Math.round(usableHours * 60);
  if (totalMinutes === 0) return "0m";

  const fullHours = Math.floor(totalMinutes / 60);
  if (fullHours >= 1) {
    return `${fullHours}h`;
  }

  return `${totalMinutes}m`;
}
