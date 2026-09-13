import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { nextTick, ref } from "vue";
import TodayView from "./TodayView.vue";
import TileApp from "./TileApp.vue";
import type { NewTask, Task } from "../types";
import { deferred, makeTask } from "../test/fixtures";
import { find, hasClass, mount, trigger } from "../test/render";

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ close: vi.fn() }),
  PhysicalPosition: class {},
}));
vi.mock("../composables/useClock", () => ({
  useClock: () => ref(new Date("2026-09-13T12:00:00")),
}));
vi.mock("../composables/useProjects", () => ({
  useProjects: () => ({ projects: ref([]), projectMap: ref({}), nameOf: () => "未分组" }),
}));
vi.mock("../composables/useTasks", () => ({
  useTasks: () => ({
    tasks: ref([]), todayTasks: ref([]), error: ref(""), create,
    complete: vi.fn(), reopen: vi.fn(), refresh: vi.fn().mockResolvedValue(undefined),
  }),
}));

const create = vi.fn<(input: NewTask) => Promise<Task>>();
const cleanup: (() => void)[] = [];

beforeEach(() => {
  vi.useFakeTimers();
  vi.setSystemTime(new Date("2026-09-13T12:00:00"));
  vi.stubGlobal("window", { setInterval, clearInterval, setTimeout, clearTimeout });
  vi.stubGlobal("document", { activeElement: null });
  vi.stubGlobal("Document", class {});
  vi.stubGlobal("ShadowRoot", class {});
  create.mockReset().mockResolvedValue(makeTask());
});

afterEach(() => {
  cleanup.splice(0).forEach((unmount) => unmount());
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

describe.each([
  ["今日视图", TodayView, "composer"],
  ["磁贴", TileApp, "tile-add"],
] as const)("%s 快速创建", (_name, View, formClass) => {
  function start() {
    const view = mount(View, {
      now: new Date("2026-09-13T12:00:00"), tasks: [], completedTasks: [], selected: null,
      onSave: vi.fn().mockResolvedValue(undefined),
      onCreate: async (input: NewTask) => { await create(input); },
    });
    cleanup.push(view.unmount);
    const form = find(view.root, (target) => hasClass(target, formClass));
    const input = find(form, (target) => target.tag === "input");
    return { form, input };
  }

  it("连续提交只创建一次，成功后保留期间输入的下一条任务", async () => {
    const response = deferred<Task>();
    create.mockReturnValueOnce(response.promise);
    const { form, input } = start();
    trigger(input, "onUpdate:modelValue", "任务 A");
    const saving = trigger(form, "onSubmit");
    trigger(form, "onSubmit");
    await nextTick();
    expect(create).toHaveBeenCalledTimes(1);
    expect(form.props["aria-busy"]).toBe(true);

    trigger(input, "onUpdate:modelValue", "任务 B");
    response.resolve(makeTask({ title: "任务 A" }));
    await saving;
    await nextTick();
    expect(input.value).toBe("任务 B");
    expect(form.props["aria-busy"]).toBe(false);

    await trigger(form, "onSubmit");
    await nextTick();
    expect(create).toHaveBeenCalledTimes(2);
    expect(create).toHaveBeenLastCalledWith(expect.objectContaining({ title: "任务 B" }));
    expect(input.value).toBe("");
  });

  it("创建失败保留原始输入并释放提交锁", async () => {
    create.mockRejectedValueOnce(new Error("创建失败"));
    const { form, input } = start();
    trigger(input, "onUpdate:modelValue", "  待重试的任务  ");
    await trigger(form, "onSubmit");
    await nextTick();
    expect(input.value).toBe("  待重试的任务  ");
    expect(form.props["aria-busy"]).toBe(false);

    await trigger(form, "onSubmit");
    await nextTick();
    expect(create).toHaveBeenCalledTimes(2);
    expect(create).toHaveBeenLastCalledWith(expect.objectContaining({ title: "待重试的任务" }));
    expect(input.value).toBe("");
  });
});
