import { useEffect, useState } from "react";
import "./App.css";
import { invoke } from "@tauri-apps/api/core";

type Window = {
  hwnd: number;
  title: string;
  process_id: number;
};

function App() {
  const [windows, setWindows] = useState<Window[]>([]);

  // useEffect(() => {
  //   async function getWindows() {
  //     const windows = await invoke("get_windows");
  //     setWindows(windows);
  //   }
  //   getWindows();
  // }, []);
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
        <button
          className="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
          onClick={async () => {
            const windows = await invoke<Window[]>("get_windows");
            console.log("windows", windows);
            setWindows(windows);
          }}
        >
          get windows
        </button>
        {windows.map((window: any) => (
          <p
            onClick={async () => {
              await invoke("focus_window", { hwnd: window.hwnd });
            }}
            className="text-white"
          >
            {window.title}
          </p>
        ))}
      </div>
    </main>
  );
}

export default App;
