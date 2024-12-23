import { useEffect, useState } from "preact/hooks";

interface Download {
  token: number;
  filename: string;
  last_used: string;
  download_time: string;
}

interface Executable {
  id: string;
  name: string;
  size: number;
}

interface UseSocketResult {
  id: string | null;
  executables: Executable[];
  downloads: Download[] | null;
  deleteDownload: (id: string) => void;
}

function useSocket(): UseSocketResult {
  const [id, setId] = useState<string | null>(null);
  const [downloads, setDownloads] = useState<Download[] | null>(null);
  const [executables, setExecutables] = useState<string | null>(null);

  function deleteDownload() {}

  useEffect(() => {
    const socket = new WebSocket(
      (window.location.protocol === "https:" ? "wss://" : "ws://") +
        (import.meta.env.DEV != undefined
          ? "localhost:5800"
          : window.location.host) +
        "/ws"
    );

    socket.onmessage = (event) => {
      const data = JSON.parse(event.data);

      if (data.type == undefined)
        throw new Error("Received message without type");

      switch (data.type) {
        case "state":
          const downloads = data.downloads as Download[];
          setId(data.session);
          setDownloads(downloads);
          break;
        default:
          console.warn("Received unknown message type", data.type);
      }
    };

    socket.onclose = () => {
      console.log("WebSocket connection closed");
    };

    return () => {
      // Close the socket when the component is unmounted
      socket.close();
    };
  }, []);

  return { id, downloads, deleteDownload };
}

export default useSocket;
