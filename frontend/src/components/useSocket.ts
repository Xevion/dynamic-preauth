import { withBackend } from "@/util";
import { useEffect, useRef, useState } from "react";
import useWebSocket, { ReadyState } from "react-use-websocket";

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
  executables: Executable[] | null;
  downloads: Download[] | null;
  buildLog: string | null;
  deleteDownload: (id: number) => void;
}

export interface UseSocketProps {
  notify?: (token: number) => void;
}

export type Status =
  | "connecting"
  | "open"
  | "closing"
  | "closed"
  | "uninstantiated";

function useSocket({ notify }: UseSocketProps): UseSocketResult {
  const { sendMessage, lastMessage, readyState } = useWebSocket(
    withBackend(
      window.location.protocol === "https:" ? "wss://" : "ws://",
      "/ws"
    ),
    {
      shouldReconnect: () => true,
    }
  );

  const [id, setId] = useState<number | null>(null);
  const [downloads, setDownloads] = useState<Download[] | null>(null);
  const [executables, setExecutables] = useState<{
    build_log: string | null;
    executables: Executable[];
  } | null>(null);

  const connectionStatus: Status = {
    [ReadyState.CONNECTING]: "connecting",
    [ReadyState.OPEN]: "open",
    [ReadyState.CLOSING]: "closing",
    [ReadyState.CLOSED]: "closed",
    [ReadyState.UNINSTANTIATED]: "uninstantiated",
  }[readyState] as Status;

  useEffect(() => {
    if (readyState === WebSocket.CLOSED || readyState === WebSocket.CLOSING) {
      setId(null);
      setDownloads(null);
      setExecutables(null);
    }
  }, [readyState]);

  useEffect(() => {
    {
      if (lastMessage == null) return;
      const data = JSON.parse(lastMessage.data);

      if (data.type == undefined)
        throw new Error("Received message without type");

      switch (data.type) {
        case "notify":
          const token = data.token as number;
          if (notify != null) notify(token);
          break;
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
    }
  }, [lastMessage]);

  function deleteDownload(download_token: number) {
    if (readyState !== WebSocket.OPEN) return;

    sendMessage(
      JSON.stringify({
        type: "delete-download-token",
        id: download_token,
      })
    );
  }

  return {
    id,
    downloads,
    executables: executables?.executables ?? null,
    buildLog: executables?.build_log ?? null,
    deleteDownload,
  };
}

export default useSocket;
