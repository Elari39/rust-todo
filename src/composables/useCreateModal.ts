import { ref } from "vue";

/** 新建任务弹窗的预填项：日历视图会带上所选日期 */
export interface CreatePrefill {
  startAt?: string | null;
  dueAt?: string | null;
  projectId?: string | null;
}

// 模块级单例：弹窗只挂在 App.vue 一处，任意视图都能打开
const open = ref(false);
const prefill = ref<CreatePrefill>({});

export function useCreateModal() {
  function openCreate(next: CreatePrefill = {}) {
    prefill.value = next;
    open.value = true;
  }

  function closeCreate() {
    open.value = false;
  }

  return { open, prefill, openCreate, closeCreate };
}
