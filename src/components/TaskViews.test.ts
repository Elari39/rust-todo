import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { nextTick, reactive, ref } from "vue";
import AllTasksView from "./AllTasksView.vue";
import TodayView from "./TodayView.vue";
import TaskDetail from "./TaskDetail.vue";
import { makeTask } from "../test/fixtures";
import { find, findAll, hasClass, mount, trigger, type TestNode } from "../test/render";

vi.mock("../composables/useClock", () => ({
  useClock: () => ref(new Date("2026-09-13T12:00:00")),
}));
vi.mock("../composables/useProjects", () => ({
  useProjects: () => ({
    projects: ref([]),
    projectMap: ref({}),
    nameOf: () => "未分组",
    load: vi.fn(),
  }),
}));

const cleanup: (() => void)[] = [];

beforeEach(() => {
  vi.stubGlobal("document", { activeElement: null });
  vi.stubGlobal("Document", class {});
  vi.stubGlobal("ShadowRoot", class {});
});

afterEach(() => {
  cleanup.splice(0).forEach((unmount) => unmount());
  vi.unstubAllGlobals();
});

async function edit(root: TestNode) {
  trigger(find(root, (target) => target.tag === "button" && target.textContent.trim() === "编辑"), "onClick");
  await nextTick();
}

function detailForm(root: TestNode) {
  return find(root, (target) => target.tag === "form" && hasClass(target, "form-grid"));
}

describe.each([
  ["今日视图", TodayView],
  ["全部任务视图", AllTasksView],
] as const)("%s 的详情保存", (_name, View) => {
  it("编辑 A 后切换 B 会结束编辑，重新编辑只提交 B 的内容", async () => {
    const a = makeTask({ id: "A", title: "任务 A", notes: "备注 A" });
    const b = makeTask({ id: "B", title: "任务 B", notes: "备注 B", priority: "high" });
    const onSave = vi.fn();
    const props = reactive({
      now: new Date("2026-09-13T12:00:00"),
      tasks: [a, b],
      completedTasks: [],
      selected: a,
      query: "",
      onCreate: vi.fn(),
      onSave,
      onSelect: (id: string) => { props.selected = id === "A" ? a : b; },
    });
    const view = mount(View, props);
    cleanup.push(view.unmount);
    await edit(view.root);
    trigger(find(detailForm(view.root), (target) => target.tag === "input"), "onUpdate:modelValue", "A 的草稿");

    trigger(find(view.root, (target) => hasClass(target, "task-item") && target.textContent.includes("任务 B")), "onClick");
    await nextTick();
    expect(findAll(view.root, (target) => hasClass(target, "form-grid"))).toHaveLength(0);
    expect(onSave).not.toHaveBeenCalled();

    await edit(view.root);
    trigger(detailForm(view.root), "onSubmit");
    expect(onSave).toHaveBeenCalledWith("B", expect.objectContaining({
      title: "任务 B",
      notes: "备注 B",
      priority: "high",
    }));
    expect(a.title).toBe("任务 A");
  });

  it("选择刚发生变化时，旧表单事件仍绑定表单所属任务", async () => {
    const a = makeTask({ id: "A", title: "任务 A" });
    const b = makeTask({ id: "B", title: "任务 B" });
    const onSave = vi.fn();
    const props = reactive({
      now: new Date("2026-09-13T12:00:00"),
      tasks: [a, b], completedTasks: [], selected: a, query: "", onCreate: vi.fn(), onSave,
    });
    const view = mount(View, props);
    cleanup.push(view.unmount);
    await edit(view.root);
    const form = detailForm(view.root);
    trigger(find(form, (target) => target.tag === "input"), "onUpdate:modelValue", "A 的修改");
    props.selected = b;
    trigger(form, "onSubmit");
    await nextTick();
    expect(onSave).toHaveBeenCalledWith("A", expect.objectContaining({ title: "A 的修改" }));
  });
});

it("同一任务后台刷新时保留草稿，空备注和时间以 null 提交", async () => {
  const props = reactive({
    task: makeTask({ notes: "旧备注", startAt: "2026-09-13T09:00:00" }),
    onSave: vi.fn(),
  });
  const view = mount(TaskDetail, props);
  cleanup.push(view.unmount);
  await edit(view.root);
  const form = detailForm(view.root);
  trigger(find(form, (target) => target.tag === "input"), "onUpdate:modelValue", "本地草稿");
  trigger(find(form, (target) => target.tag === "textarea"), "onUpdate:modelValue", "");
  findAll(form, (target) => target.props.type === "datetime-local")
    .forEach((target) => trigger(target, "onUpdate:modelValue", ""));

  props.task = { ...props.task, title: "后台标题", updatedAt: "2026-09-13T12:00:00" };
  await nextTick();
  trigger(detailForm(view.root), "onSubmit");
  expect(props.onSave).toHaveBeenCalledWith("task-1", expect.objectContaining({
    title: "本地草稿", notes: null, startAt: null, dueAt: null,
  }));
});

it("今日仅有已完成任务时显示完成数和分组，不显示空列表提示", () => {
  const completed = makeTask({ status: "completed", completedAt: "2026-09-13T11:00:00" });
  const view = mount(TodayView, {
    now: new Date("2026-09-13T12:00:00"), tasks: [], completedTasks: [completed], selected: null,
    onCreate: vi.fn(),
  });
  cleanup.push(view.unmount);
  const stats = findAll(view.root, (target) => hasClass(target, "stat-num"));
  expect(stats.map((target) => target.textContent)).toEqual(["0", "0", "1"]);
  expect(findAll(view.root, (target) => hasClass(target, "task-item"))).toHaveLength(1);
  expect(findAll(view.root, (target) => hasClass(target, "empty"))).toHaveLength(0);
});
