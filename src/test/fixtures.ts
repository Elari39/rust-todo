import type { Task } from "../types";

export function makeTask(overrides: Partial<Task> = {}): Task {
  return {
    id: "task-1",
    title: "测试任务",
    notes: null,
    status: "pending",
    priority: "normal",
    kind: "quick",
    startAt: null,
    dueAt: "2026-09-13T18:40:00",
    completedAt: null,
    projectId: null,
    notified: false,
    sortOrder: 1,
    createdAt: "2026-09-13T09:00:00",
    updatedAt: "2026-09-13T09:00:00",
    ...overrides,
  };
}

export function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}
