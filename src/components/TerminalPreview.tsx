import { useEffect, useRef } from "react";
import { FitAddon } from "@xterm/addon-fit";
import { Terminal } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";

export function TerminalPreview() {
  const hostRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!hostRef.current) {
      return;
    }

    const terminal = new Terminal({
      convertEol: true,
      cursorBlink: true,
      fontFamily: "JetBrains Mono, Consolas, monospace",
      fontSize: 13,
      theme: {
        background: "#101114",
        foreground: "#f4f4f5",
        cursor: "#14b8a6",
      },
    });
    const fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);
    terminal.open(hostRef.current);
    fitAddon.fit();
    terminal.write("unfour@workspace:~$ SSH service reserved for russh backend\r\n");
    terminal.write("unfour@workspace:~$ session streaming will use Tauri events\r\n");

    return () => terminal.dispose();
  }, []);

  return <div className="h-full min-h-[360px] overflow-hidden rounded-md bg-zinc-950 p-2" ref={hostRef} />;
}
