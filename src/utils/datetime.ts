import type { Task } from "../types";

const WEEKDAYS = ["星期日", "星期一", "星期二", "星期三", "星期四", "星期五", "星期六"];
const WEEKDAYS_SHORT = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];

export function pad(value: number): string {
  return String(value).padStart(2, "0");
}

export function parseStamp(value?: string | null): Date | null {
  if (!value) return null;
  const parsed = new Date(value.includes("T") ? value : value.replace(" ", "T"));
  return Number.isNaN(parsed.getTime()) ? null : parsed;
}

export function toStamp(date: Date): string {
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}:00`;
}

export function toInputValue(value?: string | null): string {
  const date = parseStamp(value);
  if (!date) return "";
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

export function fromInputValue(value: string): string | null {
  if (!value) return null;
  return toStamp(new Date(value));
}

export function formatClock(date: Date): string {
  return `${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

export function formatDate(value?: string | null): string {
  const date = parseStamp(value);
  if (!date) return "未设置";
  return `${date.getFullYear()}/${pad(date.getMonth() + 1)}/${pad(date.getDate())}`;
}

export function formatDateTime(value?: string | null): string {
  const date = parseStamp(value);
  if (!date) return "未设置";
  return `${formatDate(value)} ${formatClock(date)}`;
}

export function formatRange(start?: string | null, end?: string | null): string {
  const left = parseStamp(start);
  const right = parseStamp(end);
  if (!left && !right) return "未设置时间";
  if (left && right) {
    return `${formatDateTime(start)} - ${pad(right.getMonth() + 1)}/${pad(right.getDate())} ${formatClock(right)}`;
  }
  return formatDateTime(start ?? end);
}

export function weekdayName(date: Date): string {
  return WEEKDAYS[date.getDay()];
}

export function weekdayShort(date: Date): string {
  return WEEKDAYS_SHORT[date.getDay()];
}

/** 「2026年9月9日」 */
export function formatFullDate(date: Date): string {
  return `${date.getFullYear()}年${date.getMonth() + 1}月${date.getDate()}日`;
}

/** 「9月9日 09:00」，任务行右侧的紧凑时间展示 */
export function formatShortStamp(value?: string | null): string {
  const date = parseStamp(value);
  if (!date) return "未设置时间";
  return `${date.getMonth() + 1}月${date.getDate()}日 ${formatClock(date)}`;
}

export function startOfDay(date: Date): Date {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate());
}

export function isSameDay(a: Date, b: Date): boolean {
  return a.getFullYear() === b.getFullYear() && a.getMonth() === b.getMonth() && a.getDate() === b.getDate();
}

export function isTodayStamp(value?: string | null, now = new Date()): boolean {
  const date = parseStamp(value);
  return date ? isSameDay(date, now) : false;
}

export function overlapsToday(task: Task, now = new Date()): boolean {
  if (task.status === "completed") return false;
  const start = parseStamp(task.startAt);
  const due = parseStamp(task.dueAt);
  const today = startOfDay(now);
  const tomorrow = new Date(today.getTime() + 86400000);
  if (due && due >= today && due < tomorrow) return true;
  if (start && due && start < tomorrow && due >= today) return true;
  if (start && isSameDay(start, now)) return true;
  if (!start && !due && isTodayStamp(task.createdAt, now)) return true;
  return false;
}

export function isOverdue(task: Task, now = new Date()): boolean {
  if (task.status === "completed") return false;
  const due = parseStamp(task.dueAt);
  return Boolean(due && due.getTime() < now.getTime());
}

export function durationHours(start?: string | null, end?: string | null): number | null {
  const left = parseStamp(start);
  const right = parseStamp(end);
  if (!left || !right) return null;
  return Math.max(0, Math.round((right.getTime() - left.getTime()) / 36e5));
}

export function defaultDue(hours = 18, minutes = 40): string {
  const date = new Date();
  date.setDate(date.getDate() + 1);
  date.setHours(hours, minutes, 0, 0);
  return toStamp(date);
}

export function dueLabel(value?: string | null, now = new Date()): string {
  const date = parseStamp(value);
  if (!date) return "未设置截止";
  const today = startOfDay(now);
  const target = startOfDay(date);
  const diff = Math.round((target.getTime() - today.getTime()) / 86400000);
  const clock = formatClock(date);
  if (diff === 0) return `今天 ${clock}`;
  if (diff === 1) return `明天 ${clock}`;
  if (diff === -1) return `昨天 ${clock}`;
  return `${date.getMonth() + 1}月${date.getDate()}日 ${clock}`;
}

export function nearestDue(tasks: Task[]): Date | null {
  const times = tasks
    .map((task) => parseStamp(task.dueAt))
    .filter((date): date is Date => Boolean(date))
    .sort((a, b) => a.getTime() - b.getTime());
  return times[0] ?? null;
}

export function monthMatrix(anchor: Date): Date[][] {
  const first = new Date(anchor.getFullYear(), anchor.getMonth(), 1);
  const start = new Date(first);
  // 周一为一周起点（与设计图一致）：周日落到当周最后一位
  start.setDate(1 - ((first.getDay() + 6) % 7));
  const weeks: Date[][] = [];
  for (let week = 0; week < 6; week += 1) {
    const row: Date[] = [];
    for (let day = 0; day < 7; day += 1) {
      const cell = new Date(start);
      cell.setDate(start.getDate() + week * 7 + day);
      row.push(cell);
    }
    weeks.push(row);
  }
  return weeks;
}
