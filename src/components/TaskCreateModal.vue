<script setup lang="ts">
import { computed, nextTick, reactive, ref, watch } from "vue";
import { X } from "lucide-vue-next";
import { useClock } from "../composables/useClock";
import { useCreateModal } from "../composables/useCreateModal";
import { useProjects } from "../composables/useProjects";
import { useTasks } from "../composables/useTasks";
import type { TaskPriority } from "../types";
import { defaultDue, fromInputValue, toInputValue, toStamp } from "../utils/datetime";
import { t } from "../i18n";

const { open, prefill, closeCreate } = useCreateModal();
const { create } = useTasks();
const { projects, load: loadProjects } = useProjects();
const clock = useClock();
const presetBase = ref(new Date());

watch(clock, (now) => { presetBase.value = now; });

const form = reactive({
  title: "",
  projectId: "",
  priority: "normal" as TaskPriority,
  startAt: "",
  dueAt: "",
});
// 截止时间停在预设上 = 快速任务；动过项目/优先级/自定义时间 = 详细任务
const presetChosen = ref(true);
const submitting = ref(false);
const titleInput = ref<HTMLInputElement | null>(null);

// 打开时重置表单并应用预填（日历视图会带上所选日期）
watch(open, (isOpen) => {
  if (!isOpen) return;
  // 打开时立即校正日期，不必等待下一次共享时钟的 30 秒更新。
  presetBase.value = new Date();
  form.title = "";
  form.projectId = prefill.value.projectId ?? "";
  form.priority = "normal";
  form.startAt = prefill.value.startAt ?? toInputValue(toStamp(new Date()));
  form.dueAt = prefill.value.dueAt ?? toInputValue(defaultDue());
  presetChosen.value = !prefill.value.dueAt;
  if (!projects.value.length) void loadProjects();
  void nextTick(() => titleInput.value?.focus());
});

// 预设保持 18:40 时刻，只切换日期
function presetInput(offsetDays: number): string {
  const date = new Date(presetBase.value);
  date.setDate(date.getDate() + offsetDays);
  date.setHours(18, 40, 0, 0);
  return toInputValue(toStamp(date));
}

const presets = computed(() => [
  { label: t("preset.today"), value: presetInput(0) },
  { label: t("preset.tomorrow"), value: presetInput(1) },
  { label: t("preset.dayAfter"), value: presetInput(2) },
]);

async function submit() {
  const title = form.title.trim();
  if (!title || submitting.value) return;
  submitting.value = true;
  try {
    const advanced =
      form.projectId !== "" || form.priority !== "normal" || !presetChosen.value;
    await create({
      title,
      kind: advanced ? "detailed" : "quick",
      priority: form.priority,
      startAt: fromInputValue(form.startAt) ?? toStamp(new Date()),
      dueAt: fromInputValue(form.dueAt) ?? defaultDue(),
      projectId: form.projectId || null,
    });
    closeCreate();
  } catch {
    // 失败已在错误横幅展示；保留输入让用户直接重试
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <div
    v-if="open"
    class="modal-backdrop"
    @click.self="closeCreate"
    @keydown.esc="closeCreate"
  >
    <form class="modal" @submit.prevent="submit">
      <div class="modal-head">
        <h2>{{ t("modal.createTitle") }}</h2>
        <button class="icon-btn" type="button" @click="closeCreate">
          <X :size="16" />
        </button>
      </div>

      <input
        ref="titleInput"
        v-model="form.title"
        type="text"
        :placeholder="t('modal.titlePlaceholder')"
        required
      />

      <div class="modal-field">
        <span>{{ t("modal.due") }}</span>
        <div class="modal-presets">
          <button
            v-for="preset in presets"
            :key="preset.value"
            class="due-preset"
            :class="{ on: form.dueAt === preset.value }"
            type="button"
            @click="
              form.dueAt = preset.value;
              presetChosen = true;
            "
          >
            {{ preset.label }}
          </button>
        </div>
        <input v-model="form.dueAt" type="datetime-local" @input="presetChosen = false" />
      </div>

      <div class="modal-row">
        <label class="modal-field">
          <span>{{ t("modal.project") }}</span>
          <select v-model="form.projectId">
            <option value="">{{ t("common.ungrouped") }}</option>
            <option v-for="project in projects" :key="project.id" :value="project.id">
              {{ project.name }}
            </option>
          </select>
        </label>
        <label class="modal-field">
          <span>{{ t("modal.priority") }}</span>
          <select v-model="form.priority">
            <option value="low">{{ t("priority.low") }}</option>
            <option value="normal">{{ t("priority.normal") }}</option>
            <option value="high">{{ t("priority.high") }}</option>
          </select>
        </label>
      </div>

      <div class="modal-field">
        <span>{{ t("modal.start") }}</span>
        <input v-model="form.startAt" type="datetime-local" />
      </div>

      <div class="modal-actions">
        <button class="btn btn-ghost" type="button" @click="closeCreate">
          {{ t("common.cancel") }}
        </button>
        <button class="btn btn-primary" type="submit" :disabled="submitting">
          {{ submitting ? t("modal.submitting") : t("modal.submit") }}
        </button>
      </div>
    </form>
  </div>
</template>
