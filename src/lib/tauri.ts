import { invoke } from "@tauri-apps/api/core";
import { EventCallback, listen } from "@tauri-apps/api/event";
import { NativeWindow } from "./types";

export async function focusWindow(window: NativeWindow) {
  await invoke("focus_window", { window });
}

export async function hideWindow() {
  await invoke("hide_window");
}

export function addWindowsUpdatedListener(
  callback: EventCallback<NativeWindow[]>
) {
  return listen<NativeWindow[]>("windows-updated", callback);
}
