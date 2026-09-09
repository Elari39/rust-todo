import { describe, expect, it } from "vitest";
import type { Task } from "../types";
import {
  dueLabel,
  durationHours,
  formatDate,
  fromInputValue,
  isOverdue,
  monthMatrix,
  nearestDue,
  overlapsToday,
  pad,
  parseStamp,
  startOfDay,
  toInputValue,
  toStamp,
} from "./datetime";

function task(overrides: Partial<Task> = {}): Task {
  return {
    id: "t1",
    title: "任务",
    notes: null,
    status: "pending",
    priority: "normal",
    kind: "quick",
    startAt: null,
    dueAt: null,
    completedAt: null,
    projectId: null,
    notified: false,
    sortOrder: 1,
    createdAt: "2026-09-09T09:00:00",
    updatedAt: "2026-09-09T09:00:00",
    ...overrides,
  };
}

describe("pad", () => {
  it("补零到两位", () => {
    expect(pad(0)).toBe("00");
    expect(pad(5)).toBe("05");
    expect(pad(12)).toBe("12");
  });
});

describe("parseStamp", () => {
  it("解析 ISO 与空格分隔两种格式", () => {
    const expected = new Date(2026, 8, 9, 18, 40, 0).getTime();
    expect(parseStamp("2026-09-09T18:40:00")?.getTime()).toBe(expected);
    expect(parseStamp("2026-09-09 18:40:00")?.getTime()).toBe(expected);
  });

  it("空值与非法值返回 null", () => {
    expect(parseStamp(null)).toBeNull();
    expect(parseStamp("")).toBeNull();
    expect(parseStamp("not-a-date")).toBeNull();
  });
});

describe("stamp 往返", () => {
  it("toInputValue / fromInputValue 保持分钟精度", () => {
    const input = toInputValue(toStamp(new Date(2026, 8, 9, 18, 40, 30)));
    expect(input).toBe("2026-09-09T18:40");
    expect(fromInputValue(input)).toBe("2026-09-09T18:40:00");
  });

  it("空值转换", () => {
    expect(toInputValue(null)).toBe("");
    expect(fromInputValue("")).toBeNull();
  });
});

describe("formatDate", () => {
  it("输出 YYYY/MM/DD，缺省给占位文案", () => {
    expect(formatDate("2026-09-09T18:40:00")).toBe("2026/09/09");
    expect(formatDate(null)).toBe("未设置");
  });
});

describe("durationHours", () => {
  it("按小时取整，缺一端返回 null", () => {
    expect(durationHours("2026-09-09T09:00:00", "2026-09-09T11:00:00")).toBe(2);
    expect(durationHours("2026-09-09T09:00:00", "2026-09-09T09:30:00")).toBe(1);
    expect(durationHours("2026-09-09T09:00:00", "2026-09-09T09:15:00")).toBe(0);
    expect(durationHours(null, "2026-09-09T09:15:00")).toBeNull();
  });
});

describe("isOverdue / overlapsToday", () => {
  const now = new Date(2026, 8, 9, 12, 0, 0);

  it("未完成且截止已过才算超期", () => {
    expect(isOverdue(task({ dueAt: "2026-09-09T09:00:00" }), now)).toBe(true);
    expect(isOverdue(task({ dueAt: "2026-09-10T09:00:00" }), now)).toBe(false);
    expect(
      isOverdue(task({ dueAt: "2026-09-09T09:00:00", status: "completed" }), now),
    ).toBe(false);
  });

  it("截止在今天、今天开始的任务都算今日；已完成不算", () => {
    expect(overlapsToday(task({ dueAt: "2026-09-09T18:00:00" }), now)).toBe(true);
    expect(overlapsToday(task({ startAt: "2026-09-09T08:00:00" }), now)).toBe(true);
    expect(overlapsToday(task({ dueAt: "2026-09-10T18:00:00" }), now)).toBe(false);
    expect(
      overlapsToday(task({ dueAt: "2026-09-09T18:00:00", status: "completed" }), now),
    ).toBe(false);
  });
});

describe("nearestDue", () => {
  it("返回最早的截止时间，空列表返回 null", () => {
    const nearest = nearestDue([
      task({ id: "a", dueAt: "2026-09-11T09:00:00" }),
      task({ id: "b", dueAt: "2026-09-10T09:00:00" }),
    ]);
    expect(nearest?.getTime()).toBe(new Date(2026, 8, 10, 9, 0, 0).getTime());
    expect(nearestDue([])).toBeNull();
  });
});

describe("startOfDay / dueLabel", () => {
  it("startOfDay 归零到本地零点", () => {
    const day = startOfDay(new Date(2026, 8, 9, 18, 40));
    expect(day.getHours()).toBe(0);
    expect(day.getDate()).toBe(9);
  });

  it("dueLabel 用今天/明天/昨天表述相对日期", () => {
    const now = new Date(2026, 8, 9, 12, 0, 0);
    expect(dueLabel("2026-09-09T18:40:00", now)).toBe("今天 18:40");
    expect(dueLabel("2026-09-10T08:05:00", now)).toBe("明天 08:05");
    expect(dueLabel("2026-09-08T07:00:00", now)).toBe("昨天 07:00");
    expect(dueLabel(null, now)).toBe("未设置截止");
  });
});

describe("monthMatrix", () => {
  it("固定输出 6 行 7 列，从当月第一周周一起", () => {
    const weeks = monthMatrix(new Date(2026, 8, 9));
    expect(weeks).toHaveLength(6);
    for (const week of weeks) {
      expect(week).toHaveLength(7);
      expect(week[0].getDay()).toBe(1);
    }
    const offset = (new Date(2026, 8, 1).getDay() + 6) % 7;
    const first = new Date(2026, 8, 1 - offset);
    expect(weeks[0][0].getTime()).toBe(first.getTime());
  });
});
