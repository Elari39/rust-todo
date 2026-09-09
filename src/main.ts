import { createApp } from "vue";
import App from "./App.vue";
import { useTasks } from "./composables/useTasks";
import "./style.css";

const app = createApp(App);

// 组件内未捕获的异常统一进错误横幅，不再只在控制台无感消失
app.config.errorHandler = (err, _instance, info) => {
  console.error("[todo]", err, info);
  const { error } = useTasks();
  error.value = err instanceof Error ? err.message : String(err);
};

app.mount("#app");
