<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { getVersion } from "@tauri-apps/api/app";
import { Download, Pin, Power } from "lucide-vue-next";
import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
import { api } from "../api";
import { useSettings } from "../composables/useSettings";
import { useTile } from "../composables/useTile";

const { settings, loadError, load, save } = useSettings();
const { tileOpen, init, toggle: toggleTileWindow } = useTile();

const leadInput = ref("15");
const autoStart = ref(false);
const dir = ref("");
const busy = ref("");
const message = ref("");
const version = ref("");
let flashTimer = 0;

onMounted(async () => {
  await load();
  leadInput.value = String(settings.value.notificationLeadMinutes);
  autoStart.value = await isEnabled().catch(() => false);
  dir.value = await api.dataDir().catch(() => "");
  version.value = await getVersion().catch(() => "");
  void init();
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

async function saveLead() {
  busy.value = "lead";
  try {
    const saved = await save({
      ...settings.value,
      notificationLeadMinutes: Number(leadInput.value) || 0,
    });
    leadInput.value = String(saved.notificationLeadMinutes);
    flash("通知提前量已保存");
  } catch (err) {
    flash(err instanceof Error ? err.message : String(err));
  } finally {
    busy.value = "";
  }
}

async function toggleTile() {
  busy.value = "tile";
  try {
    await toggleTileWindow();
  } catch (err) {
    flash(err instanceof Error ? err.message : String(err));
  } finally {
    busy.value = "";
  }
}

async function toggleCloseToTray() {
  busy.value = "tray";
  try {
    await save({ ...settings.value, closeToTray: !settings.value.closeToTray });
    flash(settings.value.closeToTray ? "关闭窗口时将最小化到托盘" : "关闭窗口将直接退出");
  } catch (err) {
    flash(err instanceof Error ? err.message : String(err));
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
    flash(autoStart.value ? "已开启开机自启" : "已关闭开机自启");
  } catch (err) {
    flash(err instanceof Error ? err.message : String(err));
  } finally {
    busy.value = "";
  }
}

async function exportBackup() {
  busy.value = "export";
  try {
    const path = await api.exportBackup();
    flash(`已导出：${path}`);
  } catch (err) {
    flash(err instanceof Error ? err.message : String(err));
  } finally {
    busy.value = "";
  }
}
</script>

<template>
  <section class="workspace">
    <header class="hero">
      <p class="kicker">设置</p>
      <h1>偏好与数据</h1>
      <p>改动即时生效，设置保存在本机。</p>
    </header>

    <div class="panel settings-panel">
      <div class="settings-row">
        <div>
          <b>通知提前量</b>
          <p class="settings-hint">到期前多少分钟发系统通知（0–720）。</p>
        </div>
        <form class="settings-inline" @submit.prevent="saveLead">
          <input v-model="leadInput" type="number" min="0" max="720" step="5" />
          <span>分钟</span>
          <button class="btn btn-green" type="submit" :disabled="busy === 'lead'">
            {{ busy === "lead" ? "保存中…" : "保存" }}
          </button>
        </form>
      </div>

      <div class="settings-row">
        <div>
          <b>置顶磁贴</b>
          <p class="settings-hint">一个始终置顶的小窗口，随手勾掉今日待办。</p>
        </div>
        <button
          class="btn"
          :class="tileOpen ? 'btn-ghost' : 'btn-orange'"
          type="button"
          :disabled="busy === 'tile'"
          @click="toggleTile"
        >
          <Pin :size="16" />
          {{ tileOpen ? "关闭磁贴" : "打开磁贴" }}
        </button>
      </div>

      <div class="settings-row">
        <div>
          <b>关闭时最小化到托盘</b>
          <p class="settings-hint">
            开启后点关闭只是隐藏窗口（托盘左键可唤回、菜单可退出）；当前状态：{{ settings.closeToTray ? "开启" : "关闭" }}。
          </p>
        </div>
        <button
          class="btn"
          :class="settings.closeToTray ? 'btn-ghost' : 'btn-green'"
          type="button"
          :disabled="busy === 'tray'"
          @click="toggleCloseToTray"
        >
          {{ settings.closeToTray ? "改为直接退出" : "改为最小化到托盘" }}
        </button>
      </div>

      <div class="settings-row">
        <div>
          <b>开机自动启动</b>
          <p class="settings-hint">登录 Windows 后自动在后台运行。当前状态：{{ autoStart ? "已开启" : "未开启" }}。</p>
        </div>
        <button
          class="btn"
          :class="autoStart ? 'btn-ghost' : 'btn-green'"
          type="button"
          :disabled="busy === 'auto'"
          @click="toggleAutoStart"
        >
          <Power :size="16" />
          {{ autoStart ? "关闭自启" : "开启自启" }}
        </button>
      </div>

      <div class="settings-row">
        <div>
          <b>导出备份</b>
          <p class="settings-hint">任务、项目和设置打包成一个 JSON 文件，写入「文档 / TodoBackup」。</p>
        </div>
        <button class="btn btn-green" type="button" :disabled="busy === 'export'" @click="exportBackup">
          <Download :size="16" />
          {{ busy === "export" ? "导出中…" : "导出 JSON" }}
        </button>
      </div>

      <div class="settings-row">
        <div>
          <b>数据目录</b>
          <p class="settings-hint">SQLite 数据库与设置文件所在位置。</p>
        </div>
        <code class="settings-path">{{ dir || "读取中…" }}</code>
      </div>

      <div class="settings-row last">
        <div>
          <b>关于</b>
          <p class="settings-hint">Todo<template v-if="version"> v{{ version }}</template> · Tauri 2 + Vue 3 + SQLite</p>
        </div>
      </div>

      <p v-if="loadError" class="error-banner" role="alert">设置读取失败，当前显示默认值：{{ loadError }}</p>
      <p v-if="message" class="settings-message">{{ message }}</p>
    </div>
  </section>
</template>
