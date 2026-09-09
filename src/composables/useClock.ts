import { ref } from "vue";

// 模块级单例时钟：每 30 秒推进一次。
// 「今日/逾期」这类依赖 new Date() 的 computed 必须订阅它，
// 否则跨午夜、到期时刻后不会重算，UI 一直停留在旧时间的状态。
const now = ref(new Date());
let started = false;

export function useClock(intervalMs = 30_000) {
  if (!started) {
    started = true;
    window.setInterval(() => {
      now.value = new Date();
    }, intervalMs);
  }
  return now;
}
