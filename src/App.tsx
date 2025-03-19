import { useCallback, useEffect, useRef, useState } from "react";
import "./App.css";
import { type UnlistenFn } from "@tauri-apps/api/event";
import { Fzf } from "fzf";
import { cn } from "./lib/utils";
import { ChevronRight } from "lucide-react";
import {
  addWindowsUpdatedListener,
  focusWindow,
  hideWindow,
} from "./lib/tauri";
import { NativeWindow } from "./lib/types";

function App() {
  const [windows, setWindows] = useState<NativeWindow[]>([]);
  const [search, setSearch] = useState("");
  const [selectedWindow, setSelectedWindow] = useState<number>(0);

  const searchInputRef = useRef<HTMLInputElement>(null);

  const fzf = new Fzf(windows, {
    selector: (item) => item.title,
  });

  const filteredWindows = fzf.find(search).map((item) => item.item);

  function getNextWindow() {
    setSelectedWindow((prev) =>
      prev < filteredWindows.length - 1 ? prev + 1 : 0
    );
  }

  function getPreviousWindow() {
    setSelectedWindow((prev) =>
      prev > 0 ? prev - 1 : filteredWindows.length - 1
    );
  }

  const handleKeyDown = useCallback(
    async (e: KeyboardEvent) => {
      if (
        (e.ctrlKey && e.key === "k") ||
        (e.key === "Tab" && e.shiftKey) ||
        e.key === "ArrowUp"
      ) {
        getNextWindow();
      }

      if (
        (e.ctrlKey && e.key === "j") ||
        (e.key === "Tab" && !e.shiftKey) ||
        e.key === "ArrowDown"
      ) {
        getPreviousWindow();
      }

      if (e.key === "Enter") {
        if (selectedWindow < filteredWindows.length) {
          const window = filteredWindows[selectedWindow];
          await focusWindow(window.hwnd);
        }
      }

      if (e.key === "Escape") {
        await hideWindow();
      }
    },
    [filteredWindows, selectedWindow]
  );

  useEffect(() => {
    let unlisten: UnlistenFn;

    const setupListener = async () => {
      try {
        unlisten = await addWindowsUpdatedListener((event) => {
          console.log("Received Tauri event:", event.payload);
          if (searchInputRef.current) {
            searchInputRef.current.focus();
          }
          setSelectedWindow(0);
          setSearch("");
          setWindows(event.payload);
        });
      } catch (error) {
        console.error("Error setting up Tauri event listener:", error);
      }
    };

    setupListener();

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, []);

  useEffect(() => {
    setSelectedWindow(0);
  }, [search]);

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [filteredWindows, selectedWindow]);

  return (
    <div className="bg-slate-900 flex flex-col p-4 gap-2 h-screen w-screen">
      <div className="flex-1 flex gap-2">
        <div className="border border-white flex-1 h-full w-1/2 p-2 flex flex-col-reverse">
          {filteredWindows.map((window: NativeWindow, index: number) => {
            const isSelected = index === selectedWindow;
            return (
              <button
                key={window.hwnd}
                onClick={async () => {
                  await focusWindow(window.hwnd);
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
