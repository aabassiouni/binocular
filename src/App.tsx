import { useEffect, useRef, useState } from "react";
import "./App.css";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { Fzf } from "fzf";
import { cn } from "./lib/utils";
import { ChevronRight } from "lucide-react";

type Window = {
  hwnd: number;
  title: string;
  process_id: number;
  icon_base64?: string;
};

function App() {
  const [windows, setWindows] = useState<Window[]>([]);
  const [search, setSearch] = useState("");
  const [selectedWindow, setSelectedWindow] = useState<number>(0);

  const searchInputRef = useRef<HTMLInputElement>(null);

  const fzf = new Fzf(windows, {
    selector: (item) => item.title,
  });

  const filteredWindows = fzf.find(search).map((item) => item.item);

  useEffect(() => {
    let isMounted = true;

    let unlisten: UnlistenFn;

    const setupListener = async () => {
      try {
        unlisten = await listen<Window[]>("windows-updated", (event) => {
          console.log("Received Tauri event:", event.payload);
          if (isMounted) {
            if (searchInputRef.current) {
              searchInputRef.current.focus();
            }
            setSelectedWindow(0);
            setSearch("");
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

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey) {
        if (e.key === "k") {
          setSelectedWindow((prev) =>
            prev < filteredWindows.length - 1 ? prev + 1 : 0
          );
        } else if (e.key === "j") {
          setSelectedWindow((prev) =>
            prev > 0 ? prev - 1 : filteredWindows.length - 1
          );
        }
      }

      if (e.key === "Enter") {
        if (selectedWindow < filteredWindows.length) {
          const window = filteredWindows[selectedWindow];
          invoke("focus_window", { hwnd: window.hwnd });
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [filteredWindows, selectedWindow]);

  return (
    <div className="bg-slate-900 flex flex-col p-4 gap-2 h-screen w-screen">
      <div className="flex-1 flex gap-2">
        <div className="border border-white flex-1 h-full w-1/2 p-2 flex flex-col-reverse">
          {filteredWindows.map((window: Window, index: number) => {
            const isSelected = index === selectedWindow;
            return (
              <button
                key={window.hwnd}
                onClick={async () => {
                  await invoke("focus_window", { hwnd: window.hwnd });
                }}
                className={cn(
                  "text-white text-left whitespace-nowrap overflow-hidden text-ellipsis flex items-center gap-2 p-1",
                  isSelected ? "bg-slate-700" : ""
                )}
              >
                {window.icon_base64 ? (
                  <img
                    src={window.icon_base64}
                    alt=""
                    className="w-4 h-4 flex-shrink-0"
                  />
                ) : (
                  <div className="w-4 h-4 bg-gray-600 flex-shrink-0" />
                )}
                {window.title}
              </button>
            );
          })}
        </div>
        <div className="border border-white flex-1 h-full  w-1/2 p-2">
          <p className="text-white text-left whitespace-nowrap overflow-hidden text-ellipsis">
            {selectedWindow}
          </p>
        </div>
      </div>
      <div className="flex border-white items-center border bg-slate-900 text-white p-2">
        <ChevronRight className="text-white w-5 h-5 " />
        <input
          ref={searchInputRef}
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          autoFocus
          placeholder="Search..."
          className="focus:outline-none w-full bg-slate-900"
        />
      </div>
    </div>
  );
}

export default App;
