import { beforeEach, describe, expect, it, vi } from "vitest";
import { api } from "../api";
import type { AppSettings } from "../types";
import { deferred } from "../test/fixtures";

vi.mock("../api", () => ({ api: { getSettings: vi.fn(), saveSettings: vi.fn() } }));

const initial: AppSettings = { notificationLeadMinutes: 15, closeToTray: true, locale: "zh-CN" };

beforeEach(() => {
  vi.resetModules();
  vi.resetAllMocks();
  vi.mocked(api.getSettings).mockResolvedValue({ ...initial });
  vi.mocked(api.saveSettings).mockImplementation(async (settings) => ({ ...settings }));
});

describe("设置写入队列", () => {
  it("不同调用方连续保存不同字段时，后一次合并前一次成功的设置", async () => {
    const firstResponse = deferred<AppSettings>();
    vi.mocked(api.saveSettings).mockImplementationOnce(() => firstResponse.promise);
    const { useSettings } = await import("./useSettings");
    const firstStore = useSettings();
    const secondStore = useSettings();
    await firstStore.load();

    const lead = firstStore.save({ notificationLeadMinutes: 60 });
    const tray = secondStore.save({ closeToTray: false });
    await Promise.resolve();
    expect(api.saveSettings).toHaveBeenCalledTimes(1);
    expect(api.saveSettings).toHaveBeenNthCalledWith(1, { ...initial, notificationLeadMinutes: 60 });

    firstResponse.resolve({ ...initial, notificationLeadMinutes: 60 });
    await Promise.all([lead, tray]);
    expect(api.saveSettings).toHaveBeenNthCalledWith(2, {
      ...initial, notificationLeadMinutes: 60, closeToTray: false,
    });
    expect(firstStore.settings.value).toEqual({
      ...initial, notificationLeadMinutes: 60, closeToTray: false,
    });
    expect(secondStore.settings.value).toEqual(firstStore.settings.value);
  });

  it("保存失败不更新本地状态，也不会阻塞后续保存或重试", async () => {
    const firstResponse = deferred<AppSettings>();
    vi.mocked(api.saveSettings).mockImplementationOnce(() => firstResponse.promise);
    const { useSettings } = await import("./useSettings");
    const store = useSettings();
    await store.load();

    const lead = store.save({ notificationLeadMinutes: 60 });
    const failed = expect(lead).rejects.toThrow("保存失败");
    const tray = store.save({ closeToTray: false });
    firstResponse.reject(new Error("保存失败"));
    await failed;
    await tray;

    expect(api.saveSettings).toHaveBeenNthCalledWith(2, { ...initial, closeToTray: false });
    expect(store.settings.value).toEqual({ ...initial, closeToTray: false });
    await store.save({ notificationLeadMinutes: 60 });
    expect(store.settings.value).toEqual({ ...initial, notificationLeadMinutes: 60, closeToTray: false });
  });

  it("合并设置后仍执行提前量校验，并保留未提交的字段", async () => {
    const { useSettings } = await import("./useSettings");
    const store = useSettings();
    await store.load();
    await store.save({ closeToTray: false });
    await store.save({ notificationLeadMinutes: 999 });
    expect(store.settings.value).toEqual({ ...initial, notificationLeadMinutes: 720, closeToTray: false });
    await store.save({ notificationLeadMinutes: -1 });
    expect(store.settings.value).toEqual({ ...initial, notificationLeadMinutes: 0, closeToTray: false });
  });
});
