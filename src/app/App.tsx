import { BrowserRouter } from "react-router-dom";
import { AppRoutes } from "./router";
import WindowTitleBar from "../widgets/layout/WindowTitleBar";
import { AppDialogHost } from "../widgets/common/index.react";
import "./react-shell.css";

export default function App() {
  return (
    <BrowserRouter future={{ v7_startTransition: true, v7_relativeSplatPath: true }}>
      <div className="app-frame">
        <WindowTitleBar />
        <div className="app-content">
          <AppRoutes />
        </div>
        <AppDialogHost />
      </div>
    </BrowserRouter>
  );
}
