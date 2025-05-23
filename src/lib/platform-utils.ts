import { platform } from "@tauri-apps/api/os";

// Define the interface that all platform modules must implement
export interface PlatformModule {
  openTerminal: () => Promise<void>;
  getKeyboardShortcut: (action: string) => string;
  getDefaultBrowser: () => string;
  formatPath: (path: string) => string;
}

// Cache for platform module
let platformModule: PlatformModule | null = null;

export async function getPlatformModule(): Promise<PlatformModule> {
  if (platformModule) {
    return platformModule;
  }

  const currentPlatform = await platform();

  switch (currentPlatform) {
    case "win32":
      platformModule = (await import("./platforms/windows")).default;
      break;
    case "darwin":
      platformModule = (await import("./platforms/macos")).default;
      break;
    case "linux":
      platformModule = (await import("./platforms/linux")).default;
      break;
    default:
      // Fallback to linux
      platformModule = (await import("./platforms/linux")).default;
      break;
  }

  return platformModule;
}
