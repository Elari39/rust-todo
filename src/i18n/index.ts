import { ref } from "vue";
import { zhCN } from "./zh-CN";

export type Locale = "zh-CN";
export type MessageKey = keyof typeof zhCN;

const dictionaries: Record<Locale, Record<MessageKey, string>> = {
  "zh-CN": zhCN,
};

// locale 持久化在 settings.json（后端 Settings.locale），设置页切换后调用 setLocale
export const i18nLocale = ref<Locale>("zh-CN");

export function setLocale(next: Locale) {
  i18nLocale.value = next;
}

/** 静态词条：key 类型受词典约束，拼错 key 在 vue-tsc 阶段直接报错 */
export function t(key: MessageKey): string {
  return dictionaries[i18nLocale.value][key];
}

/** 带占位符的词条：词典中用 {name} 书写占位符 */
export function tf(
  key: MessageKey,
  params: Record<string, string | number>,
): string {
  let text: string = dictionaries[i18nLocale.value][key];
  for (const [name, value] of Object.entries(params)) {
    text = text.replaceAll(`{${name}}`, String(value));
  }
  return text;
}
