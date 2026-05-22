import { useReactAppUpdater } from "../../../shared/utils/appUpdater";

function RefreshIcon({ rotating = false }: { rotating?: boolean }) {
  return (
    <svg className={rotating ? "rotating" : ""} width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="23 4 23 10 17 10" />
      <polyline points="1 20 1 14 7 14" />
      <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
    </svg>
  );
}

function DeviceIcon() {
  return (
    <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <rect x="2" y="3" width="20" height="14" rx="2" ry="2" />
      <line x1="8" y1="21" x2="16" y2="21" />
      <line x1="12" y1="17" x2="12" y2="21" />
    </svg>
  );
}

function SuccessIcon() {
  return (
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
      <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
      <polyline points="22 4 12 14.01 9 11.01" />
    </svg>
  );
}

function RocketIcon() {
  return (
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.71-2.13 0-2.97a2.121 2.121 0 0 0-2.97 0z" />
      <path d="M15 15c1.46 3 3 3 3 3" />
      <path d="M9 9c3-1.46 3-3 3-3" />
      <path d="M12 12c4 4 10 4 10 4s0-6-4-10-10-4-10-4 0 6 4 10z" />
    </svg>
  );
}

export default function UpdatePanel() {
  const {
    status,
    progress,
    updateVersion,
    currentVersion,
    errorMessage,
    checkForUpdate,
    downloadAndInstall,
  } = useReactAppUpdater();
  const isChecking = status === "checking";
  const isHighlightState = ["available", "downloading", "ready"].includes(status);

  if (status === "up-to-date") {
    return (
      <section className="settings-card card-shell update-card">
        <div className="card-left">
          <div className="icon-box icon-success"><SuccessIcon /></div>
          <div className="text-content">
            <h2 className="main-title">当前已是最新版本</h2>
            <p className="sub-text">系统版本 <span className="version-tag">v{currentVersion}</span></p>
          </div>
        </div>
        <button className="btn btn-ghost" type="button" onClick={checkForUpdate} disabled={isChecking}>
          <RefreshIcon rotating={isChecking} />
          {isChecking ? "正在检查..." : "再次检查"}
        </button>
      </section>
    );
  }

  if (isHighlightState) {
    return (
      <section className="settings-card card-shell update-card state-highlight">
        <div className="card-left">
          <div className="icon-box icon-highlight"><RocketIcon /></div>
          <div className="text-content">
            <h2 className="main-title">
              {status === "available" ? "发现全新版本！" : status === "downloading" ? "正在下载全新版本..." : "更新已准备就绪！"}
              <span className="version-tag new-version-tag">{updateVersion}</span>
            </h2>
            {status === "downloading" ? (
              <div className="download-progress-area">
                <div className="progress-bar"><div className="progress-fill" style={{ width: `${progress}%` }} /></div>
                <p className="progress-info">正在下载更新，请稍候... {progress}%</p>
              </div>
            ) : (
              <p className="sub-text">
                {status === "available" ? <>当前版本 <span className="version-tag old-version">{currentVersion}</span></> : "新版本已下载完成，即将自动重启应用以应用更新"}
              </p>
            )}
          </div>
        </div>
        <div className="card-right">
          {status === "available" ? (
            <button className="btn btn-gradient" type="button" onClick={downloadAndInstall}>立即更新系统</button>
          ) : status === "downloading" ? (
            <button className="btn btn-downloading" type="button" disabled>
              <span className="btn-progress-fill" style={{ width: `${progress}%` }} />
              <span className="btn-progress-content"><RefreshIcon rotating />下载中 {progress}%</span>
            </button>
          ) : (
            <button className="btn btn-gradient btn-relaunch" type="button" disabled><RefreshIcon rotating />正在重启...</button>
          )}
        </div>
      </section>
    );
  }

  return (
    <section className="settings-card card-shell update-card">
      <div className="card-left">
        <div className="icon-box icon-default"><DeviceIcon /></div>
        <div className="text-content">
          <h2 className="main-title">系统版本与更新</h2>
          <p className="sub-text">
            当前版本：<span className="version-tag">v{currentVersion || "加载中..."}</span>
            {status === "error" ? <span className="error-badge" title={errorMessage}>(检查失败: {errorMessage})</span> : null}
          </p>
        </div>
      </div>
      <button className="btn btn-primary" type="button" onClick={checkForUpdate} disabled={isChecking}>
        <RefreshIcon rotating={isChecking} />
        {isChecking ? "正在检查..." : "检查更新"}
      </button>
    </section>
  );
}
