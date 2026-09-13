import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { nextTick, ref } from "vue";
import TaskCreateModal from "./TaskCreateModal.vue";
import { useCreateModal } from "../composables/useCreateModal";
import { find, findAll, hasClass, mount, trigger, type TestNode } from "../test/render";

vi.mock("../composables/useClock", () => ({ useClock: () => clock }));
vi.mock("../composables/useTasks", () => ({
  useTasks: () => ({ create: vi.fn().mockResolvedValue(undefined) }),
}));
vi.mock("../composables/useProjects", () => ({
  useProjects: () => ({ projects: ref([]), load: vi.fn().mockResolvedValue(undefined) }),
}));

const clock = ref(new Date("2026-09-13T23:59:50"));
const cleanup: (() => void)[] = [];

beforeEach(() => {
  vi.useFakeTimers();
  vi.setSystemTime(new Date("2026-09-13T23:59:50"));
  clock.value = new Date();
  vi.stubGlobal("document", { activeElement: null });
  vi.stubGlobal("Document", class {});
  vi.stubGlobal("ShadowRoot", class {});
  useCreateModal().closeCreate();
});

afterEach(() => {
  cleanup.splice(0).forEach((unmount) => unmount());
  useCreateModal().closeCreate();
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

function dueInput(root: TestNode) {
  return findAll(root, (target) => target.props.type === "datetime-local")[0];
}

function choosePreset(root: TestNode, label: string) {
  trigger(find(root, (target) => hasClass(target, "due-preset") && target.textContent === label), "onClick");
}

describe("新建任务的日期预设", () => {
  it("跨午夜重新打开时立即更新三个预设，不等待共享时钟刷新", async () => {
    const view = mount(TaskCreateModal);
    cleanup.push(view.unmount);
    const modal = useCreateModal();
    modal.openCreate();
    await nextTick();
    choosePreset(view.root, "今天 18:40");
    await nextTick();
    expect(dueInput(view.root).value).toBe("2026-09-13T18:40");
    modal.closeCreate();
    await nextTick();

    vi.setSystemTime(new Date("2026-09-14T00:00:05"));
    // clock 仍停在前一天，验证打开弹窗的即时校正。
    modal.openCreate();
    await nextTick();
    for (const [label, date] of [
      ["今天 18:40", "2026-09-14T18:40"],
      ["明天 18:40", "2026-09-15T18:40"],
      ["后天 18:40", "2026-09-16T18:40"],
    ]) {
      choosePreset(view.root, label);
      await nextTick();
      expect(dueInput(view.root).value).toBe(date);
    }
  });

  it("弹窗保持打开时预设随时钟更新，但不改写用户选择的日期", async () => {
    const view = mount(TaskCreateModal);
    cleanup.push(view.unmount);
    useCreateModal().openCreate();
    await nextTick();
    trigger(dueInput(view.root), "onUpdate:modelValue", "2026-10-01T09:30");
    vi.setSystemTime(new Date("2026-09-14T00:00:20"));
    clock.value = new Date();
    await nextTick();
    expect(dueInput(view.root).value).toBe("2026-10-01T09:30");
    choosePreset(view.root, "今天 18:40");
    await nextTick();
    expect(dueInput(view.root).value).toBe("2026-09-14T18:40");
  });
});
