import { reactive } from "vue";

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

export const appDialogState = reactive<AppDialogState>({
  visible: false,
  kind: "alert",
  tone: "default",
  icon: "info",
  title: "",
  summary: "",
  details: [],
  confirmText: "知道了",
  cancelText: "取消",
});

let resolver: ((value: boolean) => void) | null = null;

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
  appDialogState.visible = true;
  appDialogState.kind = kind;
  appDialogState.tone = tone;
  appDialogState.icon = options.icon ?? iconForTone(tone);
  appDialogState.title = options.title;
  appDialogState.summary = options.summary;
  appDialogState.details = options.details ?? [];
  appDialogState.confirmText = options.confirmText ?? (kind === "confirm" ? "确认" : "知道了");
  appDialogState.cancelText = options.cancelText ?? "取消";

  return new Promise<boolean>((resolve) => {
    resolver = resolve;
  });
}

export function closeAppDialog(result: boolean) {
  if (resolver) {
    resolver(result);
    resolver = null;
  }
  appDialogState.visible = false;
}

export function useAppDialog() {
  return {
    alert: (options: Omit<AppDialogOptions, "kind">) => openAppDialog({ ...options, kind: "alert" }),
    confirm: (options: Omit<AppDialogOptions, "kind">) => openAppDialog({ ...options, kind: "confirm" }),
    open: openAppDialog,
  };
}
