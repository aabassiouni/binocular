import { useEffect, useState } from "react";
import "./App.css";
import { invoke } from "@tauri-apps/api/core";

function App() {
  const [windows, setWindows] = useState<any>([]);

  useEffect(() => {
    async function getWindows() {
      const windows = await invoke("get_windows");
      setWindows(windows);
    }
    getWindows();
  }, []);

  return (
    <main className="bg-slate-900 p-4 h-screen w-screen flex justify-center items-center">
      <div className="border border-white h-full w-full flex flex-col justify-center items-center">
        {windows.map((window: any) => (
          <p className="text-white">{window}</p>
        ))}
      </div>
    </main>
  );
}

export default App;
