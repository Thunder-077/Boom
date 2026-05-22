import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

const appWindow = getCurrentWindow();

export default function WindowTitleBar() {
  const [isFocused, setIsFocused] = useState(true);
  const [isMaximized, setIsMaximized] = useState(false);

  async function refreshMaxState() {
    try {
      setIsMaximized(await appWindow.isMaximized());
    } catch {
      setIsMaximized(false);
    }
  }

  async function minimizeWindow() {
    try {
      await appWindow.minimize();
    } catch (error) {
      console.error("[window-titlebar] minimize failed", error);
    }
  }

  async function toggleMaximize() {
    try {
      await appWindow.toggleMaximize();
      await refreshMaxState();
    } catch (error) {
      console.error("[window-titlebar] toggle maximize failed", error);
    }
  }

  async function closeWindow() {
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
  }, []);

  return (
    <header className={`window-titlebar ${isFocused ? "" : "unfocused"}`}>
      <div className="drag-zone" data-tauri-drag-region>
        <span className="app-icon" aria-hidden="true">
          <img className="app-logo" src="/boom.svg" alt="" />
        </span>
        <div className="title-copy">
          <span className="app-title">Boom</span>
        </div>
      </div>
      <div className="window-controls">
        <button className="win-btn" type="button" aria-label="最小化窗口" onClick={minimizeWindow}>
          <span className="material-symbols-rounded" aria-hidden="true">
            remove
          </span>
        </button>
        <button
          className="win-btn"
          type="button"
          aria-label={isMaximized ? "还原窗口" : "最大化窗口"}
          onClick={toggleMaximize}
        >
          <span className="material-symbols-rounded" aria-hidden="true">
            {isMaximized ? "filter_none" : "crop_square"}
          </span>
        </button>
        <button className="win-btn close" type="button" aria-label="关闭窗口" onClick={closeWindow}>
          <span className="material-symbols-rounded" aria-hidden="true">
            close
          </span>
        </button>
      </div>
    </header>
  );
}
