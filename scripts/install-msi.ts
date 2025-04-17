import { exec, execSync, spawn } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

async function installMsi() {
  try {
    const { stdout, stderr } = await execAsync(
      'taskkill /IM "binocular.exe" /F'
    );
    console.log("Successfully killed existing binocular process");
  } catch (error) {
    // Ignore error if process wasn't running
    console.log("No existing binocular process found");
  }

  try {
    const { stdout, stderr } = await execAsync(
      `msiexec /package src-tauri\\target\\release\\bundle\\msi\\binocular_0.1.0_x64_en-US.msi /qb+ /l! log.txt`
    );

    if (stdout) console.log("Update output:", stdout);
    if (stderr) console.error("Update errors:", stderr);

    console.log("MSI update completed successfully");
  } catch (error) {
    console.error("Error updating MSI:", error);
    process.exit(1);
  }

  try {
    spawn(`C:\\Program Files\\binocular\\binocular.exe`, [], {
      detached: true,
      stdio: "ignore",
    });

    console.log("Application launched successfully");
  } catch (error) {
    console.error("Error launching application:", error);
    process.exit(1);
  }
  process.exit(0);
}

await installMsi();
