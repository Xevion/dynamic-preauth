import { useEffect, useState } from "preact/hooks";

interface Download {
  token: number;
  filename: string;
  last_used: string;
  download_time: string;
}

interface Executable {
  id: string;
  filename: string;
  size: number;
}

interface UseSocketResult {
  id: number | null;
  executables: Executable[] | null;
  downloads: Download[] | null;
  deleteDownload: (id: string) => void;
}

function useSocket(): UseSocketResult {
  const [id, setId] = useState<number | null>(null);
  const [downloads, setDownloads] = useState<Download[] | null>(null);
  const [executables, setExecutables] = useState<Executable[] | null>(null);

  function deleteDownload() {}

  useEffect(() => {
    const socket = new WebSocket(
      (window.location.protocol === "https:" ? "wss://" : "ws://") +
        (import.meta.env.DEV ? "localhost:5800" : window.location.host) +
        "/ws"
    );

    socket.onmessage = (event) => {
      const data = JSON.parse(event.data);

      if (data.type == undefined)
        throw new Error("Received message without type");

      switch (data.type) {
        case "state":
          setId(data.id as number);
          setDownloads(data.session.downloads as Download[]);
          break;
        case "executables":
          setExecutables(data.executables as Executable[]);
          break;
        default:
          console.warn("Received unknown message type", data.type);
      }
    };

    socket.onclose = (event) => {
      console.log("WebSocket connection closed", event);
    };

    return () => {
      // Close the socket when the component is unmounted
      console.log("Unmounting, closing WebSocket connection");
      socket.close();
    };
  }, []);

  return { id, downloads, executables, deleteDownload };
}

export default useSocket;
