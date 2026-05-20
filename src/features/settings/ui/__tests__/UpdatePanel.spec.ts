import { describe, expect, it, vi } from "vitest";
import { mount } from "@vue/test-utils";
import { ref, computed } from "vue";
import UpdatePanel from "../UpdatePanel.vue";

const mockStatus = ref("idle");
const mockProgress = ref(0);
const mockUpdateVersion = ref("");
const mockCurrentVersion = ref("0.1.2");
const mockErrorMessage = ref("");
const mockStatusLabel = computed(() => "检查更新");
const mockCheckForUpdate = vi.fn();
const mockDownloadAndInstall = vi.fn();

vi.mock("../../../../shared/utils/appUpdater", () => ({
  useAppUpdater: () => ({
    status: mockStatus,
    progress: mockProgress,
    updateVersion: mockUpdateVersion,
    currentVersion: mockCurrentVersion,
    errorMessage: mockErrorMessage,
    statusLabel: mockStatusLabel,
    checkForUpdate: mockCheckForUpdate,
    downloadAndInstall: mockDownloadAndInstall,
  }),
}));

describe("UpdatePanel", () => {
  it("renders state 1 (idle)", () => {
    mockStatus.value = "idle";
    const wrapper = mount(UpdatePanel);
    expect(wrapper.text()).toContain("系统版本与更新");
    expect(wrapper.text()).toContain("检查更新");
    expect(wrapper.find(".state-highlight").exists()).toBe(false);
  });

  it("renders state 1 (checking)", () => {
    mockStatus.value = "checking";
    const wrapper = mount(UpdatePanel);
    expect(wrapper.text()).toContain("正在检查...");
    expect(wrapper.find(".rotating").exists()).toBe(true);
  });

  it("renders state 1 (error)", () => {
    mockStatus.value = "error";
    mockErrorMessage.value = "网络连接超时";
    const wrapper = mount(UpdatePanel);
    expect(wrapper.text()).toContain("检查失败: 网络连接超时");
  });

  it("renders state 2 (up-to-date)", () => {
    mockStatus.value = "up-to-date";
    const wrapper = mount(UpdatePanel);
    expect(wrapper.text()).toContain("当前已是最新版本");
    expect(wrapper.text()).toContain("再次检查");
  });

  it("renders state 3 (available)", () => {
    mockStatus.value = "available";
    mockUpdateVersion.value = "0.2.0";
    const wrapper = mount(UpdatePanel);
    expect(wrapper.find(".state-highlight").exists()).toBe(true);
    expect(wrapper.text()).toContain("发现全新版本！");
    expect(wrapper.text()).toContain("0.2.0");
    expect(wrapper.text()).toContain("立即更新系统");
  });

  it("renders state 3 (downloading)", () => {
    mockStatus.value = "downloading";
    mockUpdateVersion.value = "0.2.0";
    mockProgress.value = 45;
    const wrapper = mount(UpdatePanel);
    expect(wrapper.text()).toContain("正在下载全新版本...");
    expect(wrapper.text()).toContain("下载中 45%");
    expect(wrapper.find(".progress-fill").attributes("style")).toContain("width: 45%");
  });

  it("renders state 3 (ready)", () => {
    mockStatus.value = "ready";
    mockUpdateVersion.value = "0.2.0";
    const wrapper = mount(UpdatePanel);
    expect(wrapper.text()).toContain("更新已准备就绪！");
    expect(wrapper.text()).toContain("正在重启...");
  });
});
