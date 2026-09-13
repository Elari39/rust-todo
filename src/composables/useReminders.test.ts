import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { defineComponent, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { isPermissionGranted, requestPermission } from "@tauri-apps/plugin-notification";
import type { Task } from "../types";
import { deferred, makeTask } from "../test/fixtures";
import { mount } from "../test/render";
import { useReminders } from "./useReminders";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/plugin-notification", () => ({
  isPermissionGranted: vi.fn(), requestPermission: vi.fn(),
}));

const cleanup: (() => void)[] = [];

beforeEach(() => {
  vi.useFakeTimers();
  vi.setSystemTime(new Date("2026-09-13T12:00:00"));
  vi.resetAllMocks();
  vi.mocked(isPermissionGranted).mockResolvedValue(true);
  vi.mocked(invoke).mockResolvedValue(undefined);
  vi.stubGlobal("window", { setInterval, clearInterval });
});

afterEach(() => {
  cleanup.splice(0).forEach((unmount) => unmount());
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

function start(tasks: Task[] = [makeTask({ dueAt: "2026-09-13T12:05:00" })]) {
  const mark = vi.fn().mockResolvedValue(undefined);
  const instance = mount(defineComponent({
    setup() {
      useReminders(ref(tasks), mark, ref(15));
      return () => null;
    },
  }));
  cleanup.push(instance.unmount);
  return mark;
}

describe("通知发送结果", () => {
  it("等待原生通知命令成功后才标记已提醒，发送期间不重复扫描", async () => {
    const response = deferred<void>();
    vi.mocked(invoke).mockReturnValueOnce(response.promise);
    const mark = start();
    await vi.advanceTimersByTimeAsync(0);
    expect(invoke).toHaveBeenCalledWith("send_notification", expect.objectContaining({
      title: expect.any(String), body: expect.stringContaining("测试任务"),
    }));
    expect(mark).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(30_000);
    expect(invoke).toHaveBeenCalledTimes(1);

    response.resolve(undefined);
    await vi.advanceTimersByTimeAsync(0);
    expect(mark).toHaveBeenCalledExactlyOnceWith(["task-1"]);
  });

  it("原生发送失败不标记，下次扫描可以重试", async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error("系统通知发送失败"));
    const mark = start();
    await vi.advanceTimersByTimeAsync(0);
    expect(mark).not.toHaveBeenCalled();

    await vi.advanceTimersByTimeAsync(30_000);
    expect(invoke).toHaveBeenCalledTimes(2);
    expect(mark).toHaveBeenCalledExactlyOnceWith(["task-1"]);
  });

  it("单条通知失败不影响其他任务，只标记成功的任务", async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error("第一条发送失败"));
    const mark = start([
      makeTask({ id: "failed", dueAt: "2026-09-13T12:05:00" }),
      makeTask({ id: "sent", dueAt: "2026-09-13T12:06:00" }),
      makeTask({ id: "completed", status: "completed", dueAt: "2026-09-13T12:05:00" }),
      makeTask({ id: "notified", notified: true, dueAt: "2026-09-13T12:05:00" }),
    ]);
    await vi.advanceTimersByTimeAsync(0);
    expect(invoke).toHaveBeenCalledTimes(2);
    expect(mark).toHaveBeenCalledExactlyOnceWith(["sent"]);
  });

  it("未获权限不发送，授权后下一轮恢复提醒", async () => {
    vi.mocked(isPermissionGranted).mockResolvedValueOnce(false);
    vi.mocked(requestPermission).mockResolvedValueOnce("denied");
    const mark = start();
    await vi.advanceTimersByTimeAsync(0);
    expect(invoke).not.toHaveBeenCalled();
    expect(mark).not.toHaveBeenCalled();

    await vi.advanceTimersByTimeAsync(30_000);
    expect(mark).toHaveBeenCalledExactlyOnceWith(["task-1"]);
  });
});
