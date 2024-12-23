import { useEffect, useState } from "preact/hooks";

interface Download {
  token: number;
  filename: string;
  last_used: string;
  download_time: string;
}

interface UseSocketResult {
  sessionId: string | null;
  downloads: Download[] | null;
  deleteDownload: (id: string) => void;
}

function useSocket(): UseSocketResult {
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [sessionDownloads, setSessionDownloads] = useState<Download[] | null>(
    null
  );

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
          setSessionId(data.session);
          setSessionDownloads(downloads);
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

  return { sessionId, sessionDownloads, deleteDownload };
}

export default useSocket;
