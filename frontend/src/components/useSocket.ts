import { useEffect, useRef, useState } from "react";

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
  deleteDownload: (id: number) => void;
}

function useSocket(): UseSocketResult {
  const [id, setId] = useState<number | null>(null);
  const [downloads, setDownloads] = useState<Download[] | null>(null);
  const [executables, setExecutables] = useState<Executable[] | null>(null);
  const socketRef = useRef<WebSocket | null>(null);
  const allowReconnectRef = useRef<boolean>(true);

  function deleteDownload(download_token: number) {
    if (socketRef.current == null) {
      console.error("Socket is null");
      return;
    } else if (socketRef.current.readyState !== WebSocket.OPEN) {
      console.error("Socket is not open", socketRef.current.readyState);
      return;
    }
    socketRef.current.send(
      JSON.stringify({
        type: "delete",
        token: download_token,
      })
    );
  }

  useEffect(() => {
    function connect() {
      const socket = new WebSocket(
        (window.location.protocol === "https:" ? "wss://" : "ws://") +
          (import.meta.env.DEV ? "localhost:5800" : window.location.host) +
          "/ws"
      );
      socketRef.current = socket;

      socket.onmessage = (event) => {
        const data = JSON.parse(event.data);

        if (data.type == undefined)
          throw new Error("Received message without type");

        switch (data.type) {
          case "state":
            setId(data.session.id as number);
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

        socketRef.current = null;
        if (allowReconnectRef.current) {
          setId(null);
          setDownloads(null);
          setExecutables(null);

          setTimeout(() => {
            connect();
          }, 3000);
        }
      };
    }

    connect();

    return () => {
      // Close the socket when the component is unmounted
      console.log("Unmounting, closing WebSocket connection");
      socketRef.current?.close();
      allowReconnectRef.current = false;
    };
  }, []);

  return { id, downloads, executables, deleteDownload };
}

export default useSocket;
