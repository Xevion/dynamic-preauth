import Badge from "@/components/Badge";
import { cn, plural, type ClassValue } from "@/util";
import { useState } from "preact/hooks";

type StatefulDemoProps = {
  class?: ClassValue;
};

type SessionData = {
  id: string;
  downloads: string[];
};

const StatefulDemo = ({ class: className }: StatefulDemoProps) => {
  const randomBits = (bits: number) =>
    Math.floor(Math.random() * 2 ** bits)
      .toString(16)
      .padStart(bits / 4, "0")
      .toUpperCase();

  const [session, setSession] = useState<SessionData | null>({
    id: "0×" + randomBits(32),
    downloads: Array.from({ length: 7 }).map(() => "0×" + randomBits(16)),
  });

  return (
    <div class={cn(className, "px-5 leading-6")}>
      <p class="mt-3 mb-3">
        This demo uses websockets to communicate between the server and the
        browser. Each download gets a unique identifier bound to the user
        session.
        <br />
        {session != null ? (
          <>
            Your session is <b class="text-teal-400 font-inter">{session.id}</b>
            . You have{" "}
            <b class="text-teal-400 font-inter">{session.downloads.length}</b>{" "}
            known {plural("download", session.downloads.length)}.
          </>
        ) : null}
      </p>
      <div class="flex flex-wrap justify-center gap-y-2.5">
        {session?.downloads.map((download) => (
          <Badge
            className="grow-0"
            onClick={function onClick() {
              const audio = new Audio("/notify.wav");
              audio.volume = 0.3;
              audio.play();
            }}
          >
            {download}
          </Badge>
        ))}
      </div>
      <div class="mt-4 p-2 bg-zinc-900/90 rounded-md border border-zinc-700">
        <p class="my-0">
          The server running this is completely ephemeral, can restart at any
          time, and purges data on regular intervals - at which point the
          executables you've downloaded will no longer function.
        </p>
      </div>
    </div>
  );
};

export default StatefulDemo;
