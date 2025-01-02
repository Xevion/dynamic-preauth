import { withBackend } from "@/util";
import { useEffect, useRef, useState } from "react";

export interface Download {
  token: number;
  filename: string;
  last_used: string;
  download_time: string;
}

export interface Executable {
  id: string;
  filename: string;
  size: number;
}

export interface UseSocketResult {
  id: number | null;
  status: Status;
  executables: Executable[] | null;
  downloads: Download[] | null;
  buildLog: string | null;
  deleteDownload: (id: number) => void;
}

export type Status = "connected" | "disconnected" | "connecting";

function useSocket(): UseSocketResult {
  const [id, setId] = useState<number | null>(null);
  const [downloads, setDownloads] = useState<Download[] | null>(null);
  const [executables, setExecutables] = useState<{
    build_log: string | null;
    executables: Executable[];
  } | null>(null);
  const [status, setStatus] = useState<Status>("connecting");

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
        type: "delete-download-token",
        token: download_token,
      })
    );
  }

  useEffect(() => {
    function connect() {
      const socket = new WebSocket(
        withBackend(
          window.location.protocol === "https:" ? "wss://" : "ws://",
          "/ws"
        )
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
            setExecutables({
              build_log: data.build_log,
              executables: data.executables as Executable[],
            });
            break;
          default:
            console.warn("Received unknown message type", data.type);
        }
      };

      socket.onclose = (event) => {
        console.warn("WebSocket connection closed", event);

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
      console.debug("Unmounting, closing WebSocket connection");
      socketRef.current?.close();
      allowReconnectRef.current = false;
    };
  }, []);

  return {
    id,
    downloads,
    status,
    executables: executables?.executables ?? null,
    buildLog: executables?.build_log,
    deleteDownload,
  };
}

export default useSocket;
