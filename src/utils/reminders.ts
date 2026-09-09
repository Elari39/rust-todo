/** 24 小时补发窗口：错过的提醒在一天内仍会补发 */
export const CATCHUP_MS = 24 * 60 * 60 * 1000;

/**
 * 提醒窗口判定：截止时刻前 leadMs 毫秒到截止后 catchupMs 毫秒之间都应提醒。
 * 提前量窗口与补发窗口都按闭区间处理（边界时刻即提醒）。
 */
export function isDueForNotification(
  dueMs: number | null,
  nowMs: number,
  leadMs: number,
  catchupMs: number = CATCHUP_MS,
): boolean {
  if (dueMs === null) return false;
  return dueMs - leadMs <= nowMs && dueMs + catchupMs >= nowMs;
}
