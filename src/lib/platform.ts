import { platform } from "@tauri-apps/api/os";

// Platform-specific function interfaces
interface PlatformFunctions {
  openFileManager: () => Promise<void>;
  getShortcutKey: () => string;
  getAppDataPath: () => Promise<string>;
  showNotification: (title: string, message: string) => Promise<void>;
}

// Windows-specific implementations
const windowsFunctions: PlatformFunctions = {
  async openFileManager() {
    const { Command } = await import("@tauri-apps/api/shell");
    await new Command("explorer", [process.cwd()]).execute();
  },

  getShortcutKey() {
    return "Ctrl";
  },

  async getAppDataPath() {
    const { appDataDir } = await import("@tauri-apps/api/path");
    return await appDataDir();
  },

  async showNotification(title: string, message: string) {
    // Windows-specific notification implementation
    console.log(`Windows notification: ${title} - ${message}`);
  },
};

// macOS-specific implementations
const macOSFunctions: PlatformFunctions = {
  async openFileManager() {
    const { Command } = await import("@tauri-apps/api/shell");
    await new Command("open", [process.cwd()]).execute();
  },

  getShortcutKey() {
    return "Cmd";
  },

  async getAppDataPath() {
    const { appDataDir } = await import("@tauri-apps/api/path");
    return await appDataDir();
  },

  async showNotification(title: string, message: string) {
    // macOS-specific notification implementation
    console.log(`macOS notification: ${title} - ${message}`);
  },
};

// Linux-specific implementations (fallback)
const linuxFunctions: PlatformFunctions = {
  async openFileManager() {
    const { Command } = await import("@tauri-apps/api/shell");
    await new Command("xdg-open", [process.cwd()]).execute();
  },

  getShortcutKey() {
    return "Ctrl";
  },

  async getAppDataPath() {
    const { appDataDir } = await import("@tauri-apps/api/path");
    return await appDataDir();
  },

  async showNotification(title: string, message: string) {
    console.log(`Linux notification: ${title} - ${message}`);
  },
};

// Cache the platform detection
let platformFunctions: PlatformFunctions | null = null;

// Main function to get platform-specific functions
export async function getPlatformFunctions(): Promise<PlatformFunctions> {
  if (platformFunctions) {
    return platformFunctions;
  }

  const currentPlatform = await platform();

  switch (currentPlatform) {
    case "win32":
      platformFunctions = windowsFunctions;
      break;
    case "darwin":
      platformFunctions = macOSFunctions;
      break;
    case "linux":
      platformFunctions = linuxFunctions;
      break;
    default:
      // Fallback to Linux functions
      platformFunctions = linuxFunctions;
      break;
  }

  return platformFunctions;
}

// Convenience exports for direct access
export async function openFileManager() {
  const functions = await getPlatformFunctions();
  return functions.openFileManager();
}

export async function getShortcutKey() {
  const functions = await getPlatformFunctions();
  return functions.getShortcutKey();
}

export async function getAppDataPath() {
  const functions = await getPlatformFunctions();
  return functions.getAppDataPath();
}

export async function showNotification(title: string, message: string) {
  const functions = await getPlatformFunctions();
  return functions.showNotification(title, message);
}

// Export current platform for conditional logic elsewhere
export async function getCurrentPlatform() {
  return await platform();
}

// Utility functions for platform checking
export async function isWindows() {
  return (await platform()) === "win32";
}

export async function isMacOS() {
  return (await platform()) === "darwin";
}

export async function isLinux() {
  return (await platform()) === "linux";
}
