import Badge from "@/components/Badge";
import DownloadButton from "@/components/DownloadButton";
import Emboldened from "@/components/Emboldened";
import useSocket from "@/components/useSocket";
import { cn, plural, toHex, type ClassValue } from "@/util";
import { useRef, useState } from "react";

type DemoProps = {
  class?: ClassValue;
};

const Demo = ({ class: className }: DemoProps) => {
  const { id, downloads, executables } = useSocket();
  // TODO: Toasts

  console.log([executables == null]);

  const [highlightedIndex, setHighlightedIndex] = useState<number | null>(null);
  const highlightedTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  function highlight(index: number) {
    setHighlightedIndex(index);

    if (highlightedTimeoutRef.current != null) {
      clearTimeout(highlightedTimeoutRef.current);
    }

    highlightedTimeoutRef.current = setTimeout(() => {
      highlightedTimeoutRef.current = null;
      setHighlightedIndex(null);
    }, 1250);
  }

  return (
    <div className={cn(className, "px-5 leading-6")}>
      <p className="mt-3 mb-3">
        This demo uses websockets to communicate between the server and the
        browser. Each download gets a unique identifier bound to the user
        session.
        <br />
        Your session is{" "}
        <Emboldened skeletonWidth="0x12345678" copyable={true}>
          {id != null ? toHex(id) : null}
        </Emboldened>
        . You have{" "}
        <Emboldened className="text-teal-400 font-inter">
          {downloads?.length ?? null}
        </Emboldened>{" "}
        known {plural("download", downloads?.length ?? 0)}.
      </p>
      <div className="flex flex-wrap justify-center gap-y-2.5">
        <DownloadButton
          disabled={executables == null}
          buildLog={"https://railway.com"}
          executables={executables}
        />
        {downloads?.map((download, i) => (
          <Badge
            className={cn(
              "transition-colors border hover:border-zinc-500 duration-100 ease-in border-transparent",
              {
                "!border-zinc-300 dark:bg-zinc-600": i === highlightedIndex,
              }
            )}
            onClick={function onClick() {
              highlight(i);
              const audio = new Audio("/notify.wav");
              audio.volume = 0.5;
              audio.play();
            }}
          >
            {toHex(download.token)}
          </Badge>
        ))}
      </div>
      <div className="mt-4 p-2 bg-zinc-900/90 rounded-md border border-zinc-700">
        <p className="my-0">
          The server running this is completely ephemeral, can restart at any
          time, and purges data on regular intervals - at which point the
          executables you've downloaded will no longer function.
        </p>
      </div>
    </div>
  );
};

export default Demo;
