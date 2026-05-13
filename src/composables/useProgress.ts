import { ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ProgressPayload } from "@/types/disk";

export function useProgress() {
  const progress = ref<ProgressPayload | null>(null);
  const unlisteners: UnlistenFn[] = [];

  async function setupListeners() {
    unlisteners.push(
      await listen<ProgressPayload>("clone-progress", (e) => {
        progress.value = e.payload;
      })
    );
    unlisteners.push(
      await listen<ProgressPayload>("restore-progress", (e) => {
        progress.value = e.payload;
      })
    );
    unlisteners.push(
      await listen<ProgressPayload>("hash-progress", (e) => {
        progress.value = e.payload;
      })
    );
    unlisteners.push(
      await listen<ProgressPayload>("verify-progress", (e) => {
        progress.value = e.payload;
      })
    );
    unlisteners.push(
      await listen<ProgressPayload>("disk2disk-progress", (e) => {
        progress.value = e.payload;
      })
    );
    unlisteners.push(
      await listen<ProgressPayload>("disk2disk-verify-progress", (e) => {
        progress.value = e.payload;
      })
    );
  }

  function clearProgress() {
    progress.value = null;
  }

  function cleanup() {
    unlisteners.forEach((u) => u());
    unlisteners.length = 0;
  }

  return {
    progress,
    setupListeners,
    clearProgress,
    cleanup,
  };
}
