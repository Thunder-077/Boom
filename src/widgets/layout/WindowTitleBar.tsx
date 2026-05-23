import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, X, Copy } from "lucide-react";
import { hasDesktopRuntime } from "../../shared/utils/desktopRuntime";

function resolveAppWindow() {
  // 浏览器调试时没有 Tauri runtime，直接跳过窗口控制绑定，避免整页白屏。
  if (!hasDesktopRuntime()) {
    return null;
  }
  try {
    return getCurrentWindow();
  } catch {
    return null;
  }
}

export default function WindowTitleBar() {
  const appWindow = resolveAppWindow();
  const [isFocused, setIsFocused] = useState(true);
  const [isMaximized, setIsMaximized] = useState(false);

  async function refreshMaxState() {
    if (!appWindow) {
      setIsMaximized(false);
      return;
    }
    try {
      setIsMaximized(await appWindow.isMaximized());
    } catch {
      setIsMaximized(false);
    }
  }

  async function minimizeWindow() {
    if (!appWindow) {
      return;
    }
    try {
      await appWindow.minimize();
    } catch (error) {
      console.error("[window-titlebar] minimize failed", error);
    }
  }

  async function toggleMaximize() {
    if (!appWindow) {
      return;
    }
    try {
      await appWindow.toggleMaximize();
      await refreshMaxState();
    } catch (error) {
      console.error("[window-titlebar] toggle maximize failed", error);
    }
  }

  async function closeWindow() {
    if (!appWindow) {
      return;
    }
    try {
      await appWindow.close();
    } catch (error) {
      console.error("[window-titlebar] close failed", error);
    }
  }

  useEffect(() => {
    let disposed = false;
    let unlistenResized: (() => void) | null = null;
    let unlistenFocusChanged: (() => void) | null = null;

    async function bindWindowEvents() {
      if (!appWindow) {
        return;
      }
      await refreshMaxState();
      unlistenResized = await appWindow.onResized(() => {
        void refreshMaxState();
      });
      unlistenFocusChanged = await appWindow.onFocusChanged((event) => {
        if (!disposed) {
          setIsFocused(event.payload);
        }
      });
    }

    void bindWindowEvents();

    return () => {
      disposed = true;
      unlistenResized?.();
      unlistenFocusChanged?.();
    };
  }, [appWindow]);

  return (
    <header className={`window-titlebar ${isFocused ? "" : "unfocused"}`}>
      <div className="drag-zone" {...(appWindow ? { "data-tauri-drag-region": true } : {})}>
        <span className="app-icon" aria-hidden="true">
          <img className="app-logo" src="/boom.svg" alt="" />
        </span>
        <div className="title-copy">
          <span className="app-title">Boom</span>
        </div>
      </div>
      {appWindow ? (
        <div className="window-controls">
          <button className="win-btn" type="button" aria-label="最小化窗口" onClick={minimizeWindow}>
            <Minus size={16} />
          </button>
          <button
            className="win-btn"
            type="button"
            aria-label={isMaximized ? "还原窗口" : "最大化窗口"}
            onClick={toggleMaximize}
          >
            {isMaximized ? <Copy size={16} /> : <Square size={16} />}
          </button>
          <button className="win-btn close" type="button" aria-label="关闭窗口" onClick={closeWindow}>
            <X size={16} />
          </button>
        </div>
      ) : null}
    </header>
  );
}
