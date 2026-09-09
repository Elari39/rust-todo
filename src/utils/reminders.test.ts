import { describe, expect, it } from "vitest";
import { CATCHUP_MS, isDueForNotification } from "./reminders";

const DUE = Date.parse("2026-09-09T18:15:00");
const LEAD = 15 * 60_000;

describe("isDueForNotification", () => {
  it("无截止时间不提醒", () => {
    expect(isDueForNotification(null, Date.now(), LEAD)).toBe(false);
  });

  it("截止前提前量窗口内提醒，窗口外不提醒", () => {
    expect(isDueForNotification(DUE, DUE - LEAD, LEAD)).toBe(true);
    expect(isDueForNotification(DUE, DUE - LEAD - 1, LEAD)).toBe(false);
  });

  it("到期后仍在补发窗口内，超窗不再提醒", () => {
    expect(isDueForNotification(DUE, DUE + CATCHUP_MS, 0)).toBe(true);
    expect(isDueForNotification(DUE, DUE + CATCHUP_MS + 1, 0)).toBe(false);
  });

  it("零提前量时截止时刻即提醒", () => {
    expect(isDueForNotification(DUE, DUE - 1, 0)).toBe(false);
    expect(isDueForNotification(DUE, DUE, 0)).toBe(true);
  });
});
