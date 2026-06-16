import { useCallback, useEffect, useRef, useState } from "react";

const CHANNEL_NAME = "dynamic-preauth-tab-coordination";
const HEARTBEAT_INTERVAL_MS = 250;
const TAB_TIMEOUT_MS = 750; // 3 missed heartbeats = dead

type AudioCapability = "unknown" | "yes" | "no";

interface TabState {
  tabId: string;
  createdAt: number;
  lastFocusAt: number;
  lastHeartbeat: number;
  canPlayAudio: AudioCapability;
}

interface TabMessage {
  type: "heartbeat" | "join" | "leave" | "audio-failed";
  tabId: string;
  createdAt: number;
  lastFocusAt: number;
  canPlayAudio: AudioCapability;
  // For audio-failed: list of tabs that already tried and failed
  failedTabs?: string[];
}

interface PendingAudioRequest {
  audio: HTMLAudioElement;
  failedTabs: string[];
  resolve: (played: boolean) => void;
}

interface UseTabCoordinationResult {
  tabId: string;
  isActiveTab: boolean;
  /**
   * Attempt to play audio. If this tab can't play, cascades to next-best tab.
   * Returns true if ANY tab successfully played the audio.
   */
  tryPlayAudio: (audio: HTMLAudioElement) => Promise<boolean>;
}

function generateTabId(): string {
  return `${Date.now()}-${Math.random().toString(36).slice(2, 11)}`;
}

/**
 * Coordinates multiple browser tabs using heartbeat-based leader election.
 *
 * - Each tab broadcasts a heartbeat every 250ms
 * - Tabs that haven't been seen in 750ms are considered dead
 * - The "active" tab is the one with the most recent focus time
 * - Audio playback cascades through tabs until one succeeds
 */
export function useTabCoordination(): UseTabCoordinationResult {
  const tabIdRef = useRef<string>(generateTabId());
  const createdAtRef = useRef<number>(Date.now());
  const lastFocusAtRef = useRef<number>(Date.now());
  const canPlayAudioRef = useRef<AudioCapability>("unknown");
  const tabsRef = useRef<Map<string, TabState>>(new Map());
  const channelRef = useRef<BroadcastChannel | null>(null);
  const pendingAudioRef = useRef<PendingAudioRequest | null>(null);
  const [isActiveTab, setIsActiveTab] = useState(true);

  // Get sorted list of live tabs (for audio cascade ordering)
  const getSortedLiveTabs = useCallback((): TabState[] => {
    const now = Date.now();
    const myTabId = tabIdRef.current;
    const tabs = tabsRef.current;

    const liveTabs: TabState[] = [];

    // Add self
    liveTabs.push({
      tabId: myTabId,
      createdAt: createdAtRef.current,
      lastFocusAt: lastFocusAtRef.current,
      lastHeartbeat: now,
      canPlayAudio: canPlayAudioRef.current,
    });

    // Add other live tabs
    for (const tab of tabs.values()) {
      if (tab.tabId === myTabId) continue;
      if (now - tab.lastHeartbeat <= TAB_TIMEOUT_MS) {
        liveTabs.push(tab);
      }
    }

    // Sort: canPlayAudio="yes" first, then by lastFocusAt DESC, then createdAt ASC
    liveTabs.sort((a, b) => {
      // Prefer tabs that can play audio
      const aCanPlay = a.canPlayAudio === "yes" ? 1 : 0;
      const bCanPlay = b.canPlayAudio === "yes" ? 1 : 0;
      if (bCanPlay !== aCanPlay) return bCanPlay - aCanPlay;

      // Then by most recent focus
      if (b.lastFocusAt !== a.lastFocusAt) {
        return b.lastFocusAt - a.lastFocusAt;
      }

      // Tiebreaker: older tab wins
      return a.createdAt - b.createdAt;
    });

    return liveTabs;
  }, []);

  // Compute if this tab should be the active/leader tab
  const computeIsLeader = useCallback((): boolean => {
    const sortedTabs = getSortedLiveTabs();
    return sortedTabs.length === 0 || sortedTabs[0].tabId === tabIdRef.current;
  }, [getSortedLiveTabs]);

  // Prune dead tabs and recompute leader status
  const pruneAndRecompute = useCallback(() => {
    const now = Date.now();
    const tabs = tabsRef.current;

    // Remove dead tabs
    for (const [tabId, tab] of tabs) {
      if (now - tab.lastHeartbeat > TAB_TIMEOUT_MS) {
        tabs.delete(tabId);
      }
    }

    // Recompute leader
    setIsActiveTab(computeIsLeader());
  }, [computeIsLeader]);

  // Broadcast current state
  const broadcastHeartbeat = useCallback(() => {
    channelRef.current?.postMessage({
      type: "heartbeat",
      tabId: tabIdRef.current,
      createdAt: createdAtRef.current,
      lastFocusAt: lastFocusAtRef.current,
      canPlayAudio: canPlayAudioRef.current,
    } satisfies TabMessage);
  }, []);

  // Update focus time and broadcast
  const updateFocus = useCallback(() => {
    lastFocusAtRef.current = Date.now();
    broadcastHeartbeat();
    setIsActiveTab(computeIsLeader());
  }, [broadcastHeartbeat, computeIsLeader]);

  // Track user interaction (enables audio in most browsers) and update focus time
  useEffect(() => {
    const markInteraction = () => {
      // A prior autoplay rejection no longer applies once the user has interacted
      // with this tab, so let it be reconsidered as an audio candidate.
      if (canPlayAudioRef.current === "no") {
        canPlayAudioRef.current = "unknown";
      }
      // Update lastFocusAt on any interaction - this is crucial for proper tab ordering
      // because visibility/focus events don't fire when clicking within an already-focused tab
      lastFocusAtRef.current = Date.now();
      broadcastHeartbeat();
      setIsActiveTab(computeIsLeader());
    };

    window.addEventListener("click", markInteraction);
    window.addEventListener("keydown", markInteraction);
    window.addEventListener("touchstart", markInteraction);

    return () => {
      window.removeEventListener("click", markInteraction);
      window.removeEventListener("keydown", markInteraction);
      window.removeEventListener("touchstart", markInteraction);
    };
  }, [broadcastHeartbeat, computeIsLeader]);

  // Attempt to play audio locally
  const attemptLocalPlay = useCallback(
    async (audio: HTMLAudioElement): Promise<boolean> => {
      try {
        await audio.play();
        canPlayAudioRef.current = "yes";
        broadcastHeartbeat(); // Announce we can play audio
        return true;
      } catch {
        canPlayAudioRef.current = "no";
        broadcastHeartbeat(); // Announce we can't play audio
        return false;
      }
    },
    [broadcastHeartbeat]
  );

  // Initialize coordination
  useEffect(() => {
    const channel = new BroadcastChannel(CHANNEL_NAME);
    channelRef.current = channel;

    // Handle messages from other tabs
    channel.onmessage = (event: MessageEvent<TabMessage>) => {
      const msg = event.data;
      const tabs = tabsRef.current;
      const myTabId = tabIdRef.current;

      if (msg.tabId === myTabId) return; // Ignore self

      if (msg.type === "leave") {
        tabs.delete(msg.tabId);
        pruneAndRecompute();
        return;
      }

      if (msg.type === "audio-failed") {
        // Another tab failed to play audio, check if we're next in line
        const failedTabs = msg.failedTabs || [];
        if (failedTabs.includes(myTabId)) {
          // We already tried and failed
          return;
        }

        // Get sorted tabs, find next candidate after all failed ones
        const sortedTabs = getSortedLiveTabs();
        const eligibleTabs = sortedTabs.filter(
          (t) => !failedTabs.includes(t.tabId)
        );

        if (eligibleTabs.length > 0 && eligibleTabs[0].tabId === myTabId) {
          const pending = pendingAudioRef.current;

          // Mark ourselves failed and hand off to the next eligible tab. If no
          // eligible tabs remain, the cascade is exhausted.
          const cascadeOnward = () => {
            const newFailedTabs = [...failedTabs, myTabId];
            channel.postMessage({
              type: "audio-failed",
              tabId: myTabId,
              createdAt: createdAtRef.current,
              lastFocusAt: lastFocusAtRef.current,
              canPlayAudio: canPlayAudioRef.current,
              failedTabs: newFailedTabs,
            } satisfies TabMessage);

            const remainingTabs = sortedTabs.filter(
              (t) => !newFailedTabs.includes(t.tabId)
            );
            if (remainingTabs.length === 0 && pending) {
              pending.resolve(false);
              pendingAudioRef.current = null;
            }
          };

          if (pending) {
            // We're next and have audio queued: try to play it.
            attemptLocalPlay(pending.audio).then((success) => {
              if (success) {
                pending.resolve(true);
                pendingAudioRef.current = null;
              } else {
                cascadeOnward();
              }
            });
          } else {
            // We're next but this tab's notification hasn't fired yet, so it has
            // no audio to play. Don't stall the chain - pass it to the next tab.
            cascadeOnward();
          }
        }
        return;
      }

      // Update or add tab state (heartbeat or join)
      tabs.set(msg.tabId, {
        tabId: msg.tabId,
        createdAt: msg.createdAt,
        lastFocusAt: msg.lastFocusAt,
        lastHeartbeat: Date.now(),
        canPlayAudio: msg.canPlayAudio,
      });

      // If this is a join, respond with our heartbeat so they know about us
      if (msg.type === "join") {
        broadcastHeartbeat();
      }

      // Recompute leader status
      pruneAndRecompute();
    };

    // Visibility and focus handlers
    const handleVisibilityChange = () => {
      if (document.visibilityState === "visible") {
        updateFocus();
      }
    };

    const handleFocus = () => {
      updateFocus();
    };

    // Best-effort leave notification
    const handleBeforeUnload = () => {
      channel.postMessage({
        type: "leave",
        tabId: tabIdRef.current,
        createdAt: createdAtRef.current,
        lastFocusAt: lastFocusAtRef.current,
        canPlayAudio: canPlayAudioRef.current,
      } satisfies TabMessage);
    };

    document.addEventListener("visibilitychange", handleVisibilityChange);
    window.addEventListener("focus", handleFocus);
    window.addEventListener("beforeunload", handleBeforeUnload);

    // Broadcast join message
    channel.postMessage({
      type: "join",
      tabId: tabIdRef.current,
      createdAt: createdAtRef.current,
      lastFocusAt: lastFocusAtRef.current,
      canPlayAudio: canPlayAudioRef.current,
    } satisfies TabMessage);

    // Start heartbeat interval
    const heartbeatInterval = setInterval(() => {
      broadcastHeartbeat();
      pruneAndRecompute();
    }, HEARTBEAT_INTERVAL_MS);

    // Initial focus claim if visible
    if (document.visibilityState === "visible") {
      updateFocus();
    }

    return () => {
      clearInterval(heartbeatInterval);
      document.removeEventListener("visibilitychange", handleVisibilityChange);
      window.removeEventListener("focus", handleFocus);
      window.removeEventListener("beforeunload", handleBeforeUnload);
      channel.close();
    };
  }, [
    attemptLocalPlay,
    broadcastHeartbeat,
    getSortedLiveTabs,
    pruneAndRecompute,
    updateFocus,
  ]);

  // Try to play audio with cascade fallback
  const tryPlayAudio = useCallback(
    async (audio: HTMLAudioElement): Promise<boolean> => {
      const sortedTabs = getSortedLiveTabs();
      const myTabId = tabIdRef.current;

      // Am I the first candidate?
      if (sortedTabs.length === 0 || sortedTabs[0].tabId === myTabId) {
        // Try to play locally first
        const success = await attemptLocalPlay(audio);
        if (success) {
          return true;
        }

        // Failed, cascade to other tabs
        if (sortedTabs.length > 1) {
          return new Promise((resolve) => {
            pendingAudioRef.current = {
              audio,
              failedTabs: [myTabId],
              resolve,
            };

            channelRef.current?.postMessage({
              type: "audio-failed",
              tabId: myTabId,
              createdAt: createdAtRef.current,
              lastFocusAt: lastFocusAtRef.current,
              canPlayAudio: canPlayAudioRef.current,
              failedTabs: [myTabId],
            } satisfies TabMessage);

            // Timeout: if no tab succeeds within 2 seconds, give up
            setTimeout(() => {
              if (pendingAudioRef.current?.resolve === resolve) {
                pendingAudioRef.current = null;
                resolve(false);
              }
            }, 2000);
          });
        }

        return false;
      }

      // Not the leader, store pending request in case we get cascaded to
      return new Promise((resolve) => {
        pendingAudioRef.current = { audio, failedTabs: [], resolve };

        // Timeout: leader should try first, if we don't hear back, assume it worked
        setTimeout(() => {
          if (pendingAudioRef.current?.resolve === resolve) {
            pendingAudioRef.current = null;
            resolve(false); // Assume leader handled it
          }
        }, 2000);
      });
    },
    [attemptLocalPlay, getSortedLiveTabs]
  );

  return {
    tabId: tabIdRef.current,
    isActiveTab,
    tryPlayAudio,
  };
}
