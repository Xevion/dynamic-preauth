import Badge from "@/components/Badge";
import { plural } from "@/util";
import { useState } from "preact/hooks";

type StatefulDemoProps = {
  class?: string;
};

type SessionData = {
  id: string;
  downloads: string[];
};

const StatefulDemo = ({ class: className }: StatefulDemoProps) => {
  const [session, setSession] = useState<SessionData | null>({
    id: "0x59AF5BC09",
    downloads: ["0xABF4", "0x1F3A", "0x2F3A"],
  });

  return (
    <div class="px-5 leading-6">
      <p class="mt-3">
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
      <div>
        {session?.downloads.map((download) => (
          <Badge
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
    </div>
  );
};

export default StatefulDemo;
