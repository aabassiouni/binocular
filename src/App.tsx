import { useEffect, useState } from "react";
import "./App.css";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { Fzf } from "fzf";

type Window = {
  hwnd: number;
  title: string;
  process_id: number;
};

function App() {
  const [windows, setWindows] = useState<Window[]>([]);
  const [search, setSearch] = useState("");

  const fzf = new Fzf(windows, {
    selector: (item) => item.title,
  });

  const filteredWindows = fzf.find(search).map((item) => item.item);

  console.log(filteredWindows);

  useEffect(() => {
    let isMounted = true;

    let unlisten: UnlistenFn;

    const setupListener = async () => {
      try {
        unlisten = await listen<Window[]>("windows-updated", (event) => {
          console.log("Received Tauri event:", event.payload);
          if (isMounted) {
            setWindows(event.payload);
          }
        });
      } catch (error) {
        console.error("Error setting up Tauri event listener:", error);
      }
    };

    setupListener();

    return () => {
      isMounted = false;

      if (unlisten) {
        unlisten();
      }
    };
  }, []);

  return (
    <div className="bg-slate-900 flex flex-col p-4 gap-2 h-screen w-screen">
      <div className="border border-white flex-1  h-max w-1/2 p-2 ">
        {filteredWindows.map((window: any) => (
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
      <div>
        <input
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          autoFocus
          placeholder="Search..."
          className="focus:outline-none border-white border bg-slate-900 text-white p-2"
        />
      </div>
    </div>
  );
}

export default App;
