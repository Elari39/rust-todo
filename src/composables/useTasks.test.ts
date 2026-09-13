import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ref } from "vue";
import { api } from "../api";
import { makeTask } from "../test/fixtures";

vi.mock("../api", () => ({
  api: { listTasks: vi.fn(), completeTask: vi.fn(), updateTask: vi.fn() },
}));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn().mockResolvedValue(() => {}) }));
vi.mock("@tauri-apps/api/window", () => ({ getCurrentWindow: () => ({ label: "main" }) }));
vi.mock("./useClock", () => ({ useClock: () => clock }));

const clock = ref(new Date("2026-09-13T12:00:00"));

beforeEach(() => {
  vi.resetModules();
  vi.clearAllMocks();
  clock.value = new Date("2026-09-13T12:00:00");
  vi.stubGlobal("window", { setTimeout, clearTimeout });
});

afterEach(() => vi.unstubAllGlobals());

describe("今日完成任务来源", () => {
  it("完成逾期任务后增加完成数，重新打开后恢复待办", async () => {
    let stored = makeTask({ dueAt: "2026-09-12T18:40:00" });
    vi.mocked(api.listTasks).mockImplementation(async () => [{ ...stored }]);
    vi.mocked(api.completeTask).mockImplementation(async () => {
      stored = { ...stored, status: "completed", completedAt: "2026-09-13T12:00:00" };
      return { ...stored };
    });
    vi.mocked(api.updateTask).mockImplementation(async () => {
      stored = { ...stored, status: "pending", completedAt: null };
      return { ...stored };
    });
    const store = (await import("./useTasks")).useTasks();
    await store.refresh();
    expect(store.todayCount.value).toBe(1);
    expect(store.todayCompletedTasks.value).toHaveLength(0);

    await store.complete(stored.id);
    expect(store.todayCompletedTasks.value.map((task) => task.id)).toEqual([stored.id]);
    expect(store.todayTasks.value).toHaveLength(0);
    expect(store.todayCount.value).toBe(0);

    await store.reopen(stored.id);
    expect(store.todayCompletedTasks.value).toHaveLength(0);
    expect(store.todayCount.value).toBe(1);
    expect(api.updateTask).toHaveBeenCalledWith(stored.id, { status: "pending" });
  });

  it("按完成日期而非计划日期统计，跨午夜自动重算", async () => {
    vi.mocked(api.listTasks).mockResolvedValue([
      makeTask({ id: "early", status: "completed", completedAt: "2026-09-13T00:00:00", dueAt: "2026-09-14T18:40:00" }),
      makeTask({ id: "yesterday", status: "completed", completedAt: "2026-09-12T23:59:59" }),
      makeTask({ id: "tomorrow", status: "completed", completedAt: "2026-09-14T00:00:00" }),
      makeTask({ id: "missing", status: "completed", completedAt: null }),
      makeTask({ id: "invalid", status: "completed", completedAt: "invalid" }),
      makeTask({ id: "pending", completedAt: "2026-09-13T09:00:00" }),
    ]);
    const store = (await import("./useTasks")).useTasks();
    await store.refresh();
    expect(store.todayCompletedTasks.value.map((task) => task.id)).toEqual(["early"]);
    expect(store.todayTasks.value.map((task) => task.id)).toEqual(["pending"]);

    clock.value = new Date("2026-09-14T00:00:00");
    expect(store.todayCompletedTasks.value.map((task) => task.id)).toEqual(["tomorrow"]);
    expect(store.todayTasks.value.map((task) => task.id)).toEqual(["pending"]);
  });

  it("完成操作失败时恢复待办，不产生虚假的完成统计", async () => {
    vi.mocked(api.listTasks).mockResolvedValue([makeTask()]);
    vi.mocked(api.completeTask).mockRejectedValue(new Error("写入失败"));
    const store = (await import("./useTasks")).useTasks();
    await store.refresh();
    await expect(store.complete("task-1")).rejects.toThrow("写入失败");
    expect(store.todayCount.value).toBe(1);
    expect(store.todayCompletedTasks.value).toHaveLength(0);
  });
});
