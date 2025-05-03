import { invoke } from "@tauri-apps/api/core";
import { EventCallback, listen } from "@tauri-apps/api/event";
import { NativeWindow } from "./types";

export async function focusWindow(window: NativeWindow) {
  await invoke("focus_window", { window });
}

export async function closeWindow(window: NativeWindow) {
  await invoke("close_window", { window });
}

export function addWindowsUpdatedListener(
  callback: EventCallback<NativeWindow[]>
) {
  return listen<NativeWindow[]>("windows-updated", callback);
}
