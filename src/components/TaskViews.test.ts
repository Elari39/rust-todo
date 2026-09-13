import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { nextTick, reactive, ref } from "vue";
import AllTasksView from "./AllTasksView.vue";
import TodayView from "./TodayView.vue";
import TaskDetail from "./TaskDetail.vue";
import { deferred, makeTask } from "../test/fixtures";
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
    const onSave = vi.fn().mockResolvedValue(undefined);
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
    const onSave = vi.fn().mockResolvedValue(undefined);
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

  it("保存期间保持表单且阻止重复提交，失败保留草稿并允许重试", async () => {
    const response = deferred<void>();
    const onSave = vi.fn().mockReturnValueOnce(response.promise).mockResolvedValue(undefined);
    const task = makeTask();
    const view = mount(View, {
      now: new Date("2026-09-13T12:00:00"), tasks: [task], completedTasks: [], selected: task,
      query: "", onCreate: vi.fn(), onSave,
    });
    cleanup.push(view.unmount);
    await edit(view.root);
    const form = detailForm(view.root);
    trigger(find(form, (target) => target.tag === "textarea"), "onUpdate:modelValue", "需要保留的长备注");
    const saving = trigger(form, "onSubmit");
    trigger(form, "onSubmit");
    await nextTick();
    expect(onSave).toHaveBeenCalledTimes(1);
    expect(find(detailForm(view.root), (target) => target.props.type === "submit").props.disabled).toBe(true);

    response.reject(new Error("数据库写入失败"));
    await saving;
    await nextTick();
    expect(find(detailForm(view.root), (target) => target.tag === "textarea").value).toBe("需要保留的长备注");
    expect(find(detailForm(view.root), (target) => target.props.type === "submit").props.disabled).toBe(false);
    await trigger(detailForm(view.root), "onSubmit");
    await nextTick();
    expect(onSave).toHaveBeenCalledTimes(2);
    expect(onSave).toHaveBeenLastCalledWith(task.id, expect.objectContaining({ notes: "需要保留的长备注" }));
    expect(findAll(view.root, (target) => hasClass(target, "form-grid"))).toHaveLength(0);
  });

  it.each(["成功", "失败"])("A 的迟到%s结果不关闭或改写 B 的编辑表单", async (outcome) => {
    const response = deferred<void>();
    const a = makeTask({ id: "A", title: "任务 A" });
    const b = makeTask({ id: "B", title: "任务 B" });
    const props = reactive({
      now: new Date("2026-09-13T12:00:00"), tasks: [a, b], completedTasks: [], selected: a,
      query: "", onCreate: vi.fn(), onSave: vi.fn().mockReturnValue(response.promise),
    });
    const view = mount(View, props);
    cleanup.push(view.unmount);
    await edit(view.root);
    const saving = trigger(detailForm(view.root), "onSubmit");
    props.selected = b;
    await nextTick();
    await edit(view.root);
    trigger(find(detailForm(view.root), (target) => target.tag === "textarea"), "onUpdate:modelValue", "B 的新草稿");

    if (outcome === "成功") response.resolve(undefined);
    else response.reject(new Error("A 保存失败"));
    await saving;
    await nextTick();
    expect(find(detailForm(view.root), (target) => target.tag === "input").value).toBe("任务 B");
    expect(find(detailForm(view.root), (target) => target.tag === "textarea").value).toBe("B 的新草稿");
    expect(find(detailForm(view.root), (target) => target.props.type === "submit").props.disabled).toBe(false);
  });
});

it("切换后重新编辑同一任务，旧保存结果不能解除新请求的提交锁", async () => {
  const first = deferred<void>();
  const second = deferred<void>();
  const a = makeTask({ id: "A" });
  const props = reactive({
    task: a,
    onSave: vi.fn().mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise),
  });
  const view = mount(TaskDetail, props);
  cleanup.push(view.unmount);
  await edit(view.root);
  const firstSave = trigger(detailForm(view.root), "onSubmit");
  props.task = makeTask({ id: "B" });
  await nextTick();
  props.task = a;
  await nextTick();
  await edit(view.root);
  trigger(find(detailForm(view.root), (target) => target.tag === "textarea"), "onUpdate:modelValue", "新一轮编辑");
  const secondSave = trigger(detailForm(view.root), "onSubmit");

  first.resolve(undefined);
  await firstSave;
  await nextTick();
  expect(find(detailForm(view.root), (target) => target.props.type === "submit").props.disabled).toBe(true);
  expect(find(detailForm(view.root), (target) => target.tag === "textarea").value).toBe("新一轮编辑");
  second.resolve(undefined);
  await secondSave;
  await nextTick();
  expect(findAll(view.root, (target) => hasClass(target, "form-grid"))).toHaveLength(0);
});

it("同一任务后台刷新时保留草稿，空备注和时间以 null 提交", async () => {
  const props = reactive({
    task: makeTask({ notes: "旧备注", startAt: "2026-09-13T09:00:00" }),
    onSave: vi.fn().mockResolvedValue(undefined),
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
    onCreate: vi.fn(), onSave: vi.fn().mockResolvedValue(undefined),
  });
  cleanup.push(view.unmount);
  const stats = findAll(view.root, (target) => hasClass(target, "stat-num"));
  expect(stats.map((target) => target.textContent)).toEqual(["0", "0", "1"]);
  expect(findAll(view.root, (target) => hasClass(target, "task-item"))).toHaveLength(1);
  expect(findAll(view.root, (target) => hasClass(target, "empty"))).toHaveLength(0);
});
