<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { getVersion } from "@tauri-apps/api/app";
import { open } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
import { Download, Upload } from "lucide-vue-next";
import { api } from "../api";
import { useProjects } from "../composables/useProjects";
import { useSettings } from "../composables/useSettings";
import { useTasks } from "../composables/useTasks";
import { useTile } from "../composables/useTile";
import { setLocale, t, tf, type Locale } from "../i18n";

const { settings, loadError, load, save } = useSettings();
const { tileOpen, init, toggle: toggleTileWindow } = useTile();
const { refresh: refreshTasks } = useTasks();
const { refresh: refreshProjects } = useProjects();

const leadInput = ref("15");
const autoStart = ref(false);
const dir = ref("");
const busy = ref("");
const message = ref("");
const version = ref("");
let flashTimer = 0;

// 检查更新：拉 GitHub Releases 最新 tag 与本地版本比对（不做自动安装）
const REPO = "Elari39/rust-todo";
const RELEASES_URL = `https://github.com/${REPO}/releases/latest`;
const updatePhase = ref<"idle" | "checking" | "latest" | "found" | "error">("idle");
const updateTag = ref("");

onMounted(async () => {
  await load();
  leadInput.value = String(settings.value.notificationLeadMinutes);
  autoStart.value = await isEnabled().catch(() => false);
  dir.value = await api.dataDir().catch(() => "");
  version.value = await getVersion().catch(() => "");
  void init().catch(() => {});
});

onUnmounted(() => {
  window.clearTimeout(flashTimer);
});

function flash(text: string) {
  message.value = text;
  window.clearTimeout(flashTimer);
  flashTimer = window.setTimeout(() => {
    message.value = "";
  }, 3000);
}

function fail(err: unknown) {
  return err instanceof Error ? err.message : String(err);
}

async function saveLead() {
  busy.value = "lead";
  try {
    const saved = await save({
      ...settings.value,
      notificationLeadMinutes: Number(leadInput.value) || 0,
    });
    leadInput.value = String(saved.notificationLeadMinutes);
    flash(t("set.leadSaved"));
  } catch (err) {
    flash(fail(err));
  } finally {
    busy.value = "";
  }
}

async function changeLocale(event: Event) {
  const value = (event.target as HTMLSelectElement).value as Locale;
  busy.value = "locale";
  try {
    await save({ ...settings.value, locale: value });
    setLocale(value);
  } catch (err) {
    flash(fail(err));
  } finally {
    busy.value = "";
  }
}

async function toggleTile() {
  busy.value = "tile";
  try {
    await toggleTileWindow();
  } catch (err) {
    flash(fail(err));
  } finally {
    busy.value = "";
  }
}

async function toggleCloseToTray() {
  busy.value = "tray";
  try {
    await save({ ...settings.value, closeToTray: !settings.value.closeToTray });
  } catch (err) {
    flash(fail(err));
  } finally {
    busy.value = "";
  }
}

async function toggleAutoStart() {
  busy.value = "auto";
  try {
    if (autoStart.value) {
      await disable();
      autoStart.value = false;
    } else {
      await enable();
      autoStart.value = true;
    }
  } catch (err) {
    flash(fail(err));
  } finally {
    busy.value = "";
  }
}

async function exportBackup() {
  busy.value = "export";
  try {
    const path = await api.exportBackup();
    flash(tf("set.exported", { path }));
  } catch (err) {
    flash(fail(err));
  } finally {
    busy.value = "";
  }
}

async function importBackup() {
  if (!window.confirm(t("set.importConfirm"))) return;
  busy.value = "import";
  try {
    const file = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (typeof file !== "string") return;
    const summary = await api.importBackup(file);
    flash(tf("set.imported", { s: summary }));
    // 导入整体替换了任务和项目，两份单例状态都要重拉
    await Promise.all([refreshTasks(), refreshProjects()]);
  } catch (err) {
    flash(fail(err));
  } finally {
    busy.value = "";
  }
}

function compareSemver(a: string, b: string): number {
  const pa = a.split(".").map(Number);
  const pb = b.split(".").map(Number);
  for (let index = 0; index < 3; index += 1) {
    const diff = (pa[index] || 0) - (pb[index] || 0);
    if (diff) return Math.sign(diff);
  }
  return 0;
}

async function checkUpdate() {
  updatePhase.value = "checking";
  try {
    const res = await fetch(`https://api.github.com/repos/${REPO}/releases/latest`, {
      headers: { Accept: "application/vnd.github+json" },
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const data = await res.json();
    const tag = String(data.tag_name ?? "").replace(/^v/i, "");
    if (tag && compareSemver(tag, version.value || "0.0.0") > 0) {
      updateTag.value = tag;
      updatePhase.value = "found";
    } else {
      updatePhase.value = "latest";
    }
  } catch {
    updatePhase.value = "error";
  }
}
</script>

<template>
  <section class="workspace">
    <div class="board solo">
      <div class="page-card settings-panel">
        <header class="page-head">
          <div>
            <h1 class="page-title">{{ t("set.title") }}</h1>
            <p class="page-sub">{{ t("set.subtitle") }}</p>
          </div>
        </header>

        <div class="page-scroll settings-scroll">
          <h3 class="settings-section">{{ t("set.general") }}</h3>

          <div class="settings-row">
            <div>
              <b>{{ t("set.language") }}</b>
              <p class="settings-hint">{{ t("set.languageHint") }}</p>
            </div>
            <select
              class="select"
              :value="settings.locale"
              :disabled="busy === 'locale'"
              @change="changeLocale"
            >
              <option value="zh-CN">简体中文</option>
            </select>
          </div>

          <div class="settings-row">
            <div>
              <b>{{ t("set.lead") }}</b>
              <p class="settings-hint">{{ t("set.leadHint") }}</p>
            </div>
            <form class="settings-inline" @submit.prevent="saveLead">
              <input v-model="leadInput" type="number" min="0" max="720" step="5" />
              <span>{{ t("set.leadUnit") }}</span>
              <button class="btn btn-primary" type="submit" :disabled="busy === 'lead'">
                {{ busy === "lead" ? t("set.saving") : t("common.save") }}
              </button>
            </form>
          </div>

          <div class="settings-row">
            <div>
              <b>{{ t("set.autostart") }}</b>
              <p class="settings-hint">
                {{ tf("set.autostartHint", { s: autoStart ? t("set.on") : t("set.off") }) }}
              </p>
            </div>
            <button
              class="switch"
              :class="{ on: autoStart }"
              type="button"
              role="switch"
              :aria-checked="autoStart"
              :disabled="busy === 'auto'"
              @click="toggleAutoStart"
            />
          </div>

          <div class="settings-row">
            <div>
              <b>{{ t("set.tile") }}</b>
              <p class="settings-hint">{{ t("set.tileHint") }}</p>
            </div>
            <button
              class="switch"
              :class="{ on: tileOpen }"
              type="button"
              role="switch"
              :aria-checked="tileOpen"
              :disabled="busy === 'tile'"
              @click="toggleTile"
            />
          </div>

          <div class="settings-row">
            <div>
              <b>{{ t("set.closeTray") }}</b>
              <p class="settings-hint">
                {{ tf("set.closeTrayHint", { s: settings.closeToTray ? t("set.on") : t("set.off") }) }}
              </p>
            </div>
            <button
              class="switch"
              :class="{ on: settings.closeToTray }"
              type="button"
              role="switch"
              :aria-checked="settings.closeToTray"
              :disabled="busy === 'tray'"
              @click="toggleCloseToTray"
            />
          </div>

          <h3 class="settings-section">{{ t("set.data") }}</h3>

          <div class="settings-row">
            <div>
              <b>{{ t("set.dataDir") }}</b>
              <p class="settings-hint">{{ t("set.dataDirHint") }}</p>
            </div>
            <code class="settings-path">{{ dir || t("set.loading") }}</code>
          </div>

          <div class="settings-row">
            <div>
              <b>{{ t("set.export") }}</b>
              <p class="settings-hint">{{ t("set.exportHint") }}</p>
            </div>
            <button class="btn btn-primary" type="button" :disabled="busy === 'export'" @click="exportBackup">
              <Download :size="15" />
              {{ busy === "export" ? t("set.exporting") : t("set.exportBtn") }}
            </button>
          </div>

          <div class="settings-row">
            <div>
              <b>{{ t("set.import") }}</b>
              <p class="settings-hint">{{ t("set.importHint") }}</p>
            </div>
            <button class="btn btn-ghost" type="button" :disabled="busy === 'import'" @click="importBackup">
              <Upload :size="15" />
              {{ busy === "import" ? t("set.importing") : t("set.importBtn") }}
            </button>
          </div>

          <h3 class="settings-section">{{ t("set.about") }}</h3>

          <div class="settings-row last">
            <div>
              <b>{{ t("set.version") }}</b>
              <p class="settings-hint">v{{ version || "?" }} · {{ t("set.techStack") }}</p>
            </div>
            <div class="update-result">
              <span v-if="updatePhase === 'latest'" class="update-tag">
                {{ tf("set.upToDate", { v: version }) }}
              </span>
              <template v-else-if="updatePhase === 'found'">
                <span class="update-tag">{{ tf("set.updateFound", { v: updateTag }) }}</span>
                <button class="btn btn-primary btn-sm" type="button" @click="openUrl(RELEASES_URL)">
                  {{ t("set.openRelease") }}
                </button>
              </template>
              <span v-else-if="updatePhase === 'error'" class="settings-hint">{{ t("set.checkFailed") }}</span>
              <button
                class="btn btn-ghost"
                type="button"
                :disabled="updatePhase === 'checking'"
                @click="checkUpdate"
              >
                {{ updatePhase === "checking" ? t("set.checking") : t("set.checkUpdate") }}
              </button>
            </div>
          </div>
        </div>

        <p v-if="loadError" class="error-banner" role="alert">
          {{ tf("set.loadError", { e: loadError }) }}
        </p>
        <p v-if="message" class="settings-message">{{ message }}</p>
      </div>
    </div>
  </section>
</template>
