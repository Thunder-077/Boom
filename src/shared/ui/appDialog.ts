import { reactive } from "vue";
import { useSyncExternalStore } from "react";

export type AppDialogTone = "default" | "danger" | "success" | "warning";
export type AppDialogKind = "alert" | "confirm";

export interface AppDialogOptions {
  kind?: AppDialogKind;
  tone?: AppDialogTone;
  icon?: string;
  title: string;
  summary: string;
  details?: string[];
  confirmText?: string;
  cancelText?: string;
}

interface AppDialogState {
  visible: boolean;
  kind: AppDialogKind;
  tone: AppDialogTone;
  icon: string;
  title: string;
  summary: string;
  details: string[];
  confirmText: string;
  cancelText: string;
}

const defaultAppDialogState: AppDialogState = {
  visible: false,
  kind: "alert",
  tone: "default",
  icon: "info",
  title: "",
  summary: "",
  details: [],
  confirmText: "知道了",
  cancelText: "取消",
};

export const appDialogState = reactive<AppDialogState>({ ...defaultAppDialogState });
let currentDialogState: AppDialogState = { ...defaultAppDialogState };
const listeners = new Set<() => void>();

let resolver: ((value: boolean) => void) | null = null;

function notifyDialogState() {
  Object.assign(appDialogState, currentDialogState);
  for (const listener of listeners) {
    listener();
  }
}

function iconForTone(tone: AppDialogTone) {
  if (tone === "danger") return "warning";
  if (tone === "success") return "check_circle";
  if (tone === "warning") return "error";
  return "info";
}

function openAppDialog(options: AppDialogOptions) {
  if (resolver) {
    resolver(false);
    resolver = null;
  }

  const kind = options.kind ?? "alert";
  const tone = options.tone ?? "default";
  currentDialogState = {
    visible: true,
    kind,
    tone,
    icon: options.icon ?? iconForTone(tone),
    title: options.title,
    summary: options.summary,
    details: options.details ?? [],
    confirmText: options.confirmText ?? (kind === "confirm" ? "确认" : "知道了"),
    cancelText: options.cancelText ?? "取消",
  };
  notifyDialogState();

  return new Promise<boolean>((resolve) => {
    resolver = resolve;
  });
}

export function closeAppDialog(result: boolean) {
  if (resolver) {
    resolver(result);
    resolver = null;
  }
  currentDialogState = {
    ...currentDialogState,
    visible: false,
  };
  notifyDialogState();
}

export function subscribeAppDialog(listener: () => void) {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

export function getAppDialogState() {
  return currentDialogState;
}

export function useReactAppDialogState() {
  return useSyncExternalStore(subscribeAppDialog, getAppDialogState, getAppDialogState);
}

export function useAppDialog() {
  return {
    alert: (options: Omit<AppDialogOptions, "kind">) => openAppDialog({ ...options, kind: "alert" }),
    confirm: (options: Omit<AppDialogOptions, "kind">) => openAppDialog({ ...options, kind: "confirm" }),
    open: openAppDialog,
  };
}
