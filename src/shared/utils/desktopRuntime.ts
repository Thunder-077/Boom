/**
 * 浏览器预览环境没有注入 Tauri runtime。
 * 统一用这个判断做降级，避免页面在本地预览时被桌面端 API 打断。
 */
export function hasDesktopRuntime() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}
