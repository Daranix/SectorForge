<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { confirm } from "@tauri-apps/plugin-dialog";
import TabBar from "./components/TabBar.vue";
import ClonePanel from "./components/ClonePanel.vue";
import RestorePanel from "./components/RestorePanel.vue";
import DiskToDiskPanel from "./components/DiskToDiskPanel.vue";
import VerifyPanel from "./components/VerifyPanel.vue";
import ProgressDisplay from "./components/ProgressDisplay.vue";
import { useProgress } from "@/composables/useProgress";
import { listDisks, cancelOperation, isElevated, restartAsAdmin } from "@/composables/useCloneApi";
import type { DiskInfo, OperationMode } from "@/types/disk";

const mode = ref<OperationMode>("clone");
const disks = ref<DiskInfo[]>([]);
const loadingDisks = ref(false);
const diskError = ref("");
const isAdmin = ref(true);
const showAdminWarning = ref(false);

const isRunning = ref(false);
const operationError = ref("");
const operationSuccess = ref("");
const hashResult = ref("");

const clonePanelRef = ref<InstanceType<typeof ClonePanel>>();
const restorePanelRef = ref<InstanceType<typeof RestorePanel>>();
const diskToDiskPanelRef = ref<InstanceType<typeof DiskToDiskPanel>>();
const verifyPanelRef = ref<InstanceType<typeof VerifyPanel>>();

const { progress, setupListeners, clearProgress, cleanup } = useProgress();

const canCancel = computed(() => isRunning.value);

let unlistenClose: (() => void) | null = null;

async function setupCloseHandler() {
  const win = getCurrentWindow();
  unlistenClose = await win.onCloseRequested(async (event) => {
    if (isRunning.value) {
      event.preventDefault();
      const shouldClose = await confirm(
        "An operation is currently running. Closing now may leave the disk in an inconsistent state.\n\nDo you want to cancel the operation and close SectorForge?",
        { title: "Operation in progress", kind: "warning" }
      );
      if (shouldClose) {
        try {
          await cancelOperation();
          await new Promise((r) => setTimeout(r, 500));
        } catch {
          // ignore
        }
        await win.close();
      }
    }
  });
}

async function checkElevation() {
  try {
    isAdmin.value = await isElevated();
    showAdminWarning.value = !isAdmin.value;
  } catch {
    isAdmin.value = false;
    showAdminWarning.value = true;
  }
}

async function loadDisks() {
  loadingDisks.value = true;
  diskError.value = "";
  try {
    const result = await listDisks();
    disks.value = result;
  } catch (e) {
    diskError.value = String(e);
    disks.value = [];
  } finally {
    loadingDisks.value = false;
  }
}

function handleStart(promise: Promise<unknown>) {
  clearProgress();
  operationError.value = "";
  operationSuccess.value = "";
  hashResult.value = "";
  isRunning.value = true;

  promise
    .then(() => {
      operationSuccess.value =
        mode.value === "clone"
          ? "Clone completed successfully!"
          : mode.value === "restore"
          ? "Restore completed successfully!"
          : mode.value === "disk2disk"
          ? "Disk-to-disk copy completed successfully!"
          : "";
    })
    .catch((e) => {
      operationError.value = String(e);
    })
    .finally(() => {
      isRunning.value = false;
    });
}

function handleHash(hash: string) {
  hashResult.value = hash;
  operationSuccess.value = `SHA-256: ${hash}`;
}

async function handleRestartAsAdmin() {
  try {
    await restartAsAdmin();
  } catch (e) {
    diskError.value = String(e);
  }
}

async function handleCancel() {
  try {
    await cancelOperation();
    operationError.value = "Operation cancelled";
  } catch (e) {
    console.error("Cancel error:", e);
  }
}

function resetAllPanels() {
  clearProgress();
  operationError.value = "";
  operationSuccess.value = "";
  hashResult.value = "";
  clonePanelRef.value?.reset();
  restorePanelRef.value?.reset();
  diskToDiskPanelRef.value?.reset();
  verifyPanelRef.value?.reset();
}

watch(mode, () => {
  resetAllPanels();
});

onMounted(() => {
  checkElevation();
  loadDisks();
  setupListeners();
  setupCloseHandler();
});

onUnmounted(() => {
  cleanup();
  if (unlistenClose) {
    unlistenClose();
  }
});
</script>

<template>
  <div class="app">
    <header class="header">
      <div class="logo">
        <svg width="28" height="28" viewBox="0 0 32 32" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="16" cy="16" r="14" opacity="0.9"/>
          <circle cx="16" cy="16" r="9" stroke-width="1.5" opacity="0.7"/>
          <circle cx="16" cy="16" r="4" stroke-width="1.5" opacity="0.5"/>
          <circle cx="16" cy="16" r="1.5" fill="currentColor" stroke="none"/>
          <path d="M22 16 L26 16 M24 14 L26 16 L24 18" stroke-linecap="round" stroke-linejoin="round" opacity="0.9"/>
        </svg>
        <h1>SectorForge</h1>
      </div>
      <span class="subtitle">Disk Cloner</span>
    </header>

    <TabBar v-model="mode" :disabled="isRunning" />

    <div class="content">
      <div v-if="showAdminWarning" class="admin-warning">
        <strong>Administrator privileges required</strong>
        <p>
          Physical disk access is restricted on this system.
          Please close SectorForge and restart it as Administrator
          (right-click the executable → "Run as administrator").
        </p>
        <button class="btn btn-primary admin-btn" @click="handleRestartAsAdmin">
          Restart as Administrator
        </button>
      </div>

      <button class="btn btn-secondary refresh-btn" @click="loadDisks" :disabled="loadingDisks">
        {{ loadingDisks ? "Loading..." : "Refresh Disks" }}
      </button>

      <div v-if="diskError" class="error">{{ diskError }}</div>
      <div v-else-if="!loadingDisks && disks.length === 0 && !showAdminWarning" class="empty">
        No physical disks found. Click "Refresh Disks" to try again.
      </div>

      <ClonePanel
        v-if="mode === 'clone'"
        ref="clonePanelRef"
        :disks="disks"
        :is-running="isRunning"
        @start="handleStart"
      />

      <RestorePanel
        v-if="mode === 'restore'"
        ref="restorePanelRef"
        :disks="disks"
        :is-running="isRunning"
        @start="handleStart"
      />

      <DiskToDiskPanel
        v-if="mode === 'disk2disk'"
        ref="diskToDiskPanelRef"
        :disks="disks"
        :is-running="isRunning"
        @start="handleStart"
      />

      <VerifyPanel
        v-if="mode === 'verify'"
        ref="verifyPanelRef"
        :is-running="isRunning"
        @start="handleStart"
        @hash="handleHash"
      />

      <ProgressDisplay v-if="isRunning || progress" :progress="progress" />

      <div v-if="operationSuccess && !isRunning" class="success">{{ operationSuccess }}</div>
      <div v-if="operationError && !isRunning" class="error">{{ operationError }}</div>

      <div class="actions" v-if="canCancel">
        <button class="btn btn-danger" @click="handleCancel">Cancel</button>
      </div>
    </div>
  </div>
</template>

<style>
.app {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 24px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-secondary);
}

.header .logo {
  display: flex;
  align-items: center;
  gap: 10px;
}

.header .logo h1 {
  font-size: 18px;
  font-weight: 700;
  letter-spacing: -0.02em;
}

.header .subtitle {
  font-size: 13px;
  color: var(--text-secondary);
  font-weight: 500;
}

.content {
  flex: 1;
  padding: 20px 24px;
  overflow-y: auto;
}

.refresh-btn {
  margin-bottom: 16px;
}

.actions {
  display: flex;
  gap: 10px;
  margin-top: 8px;
  margin-bottom: 16px;
}

.admin-warning {
  padding: 14px 16px;
  background: var(--warning-bg);
  border: 1px solid var(--warning);
  border-radius: var(--radius-sm);
  margin-bottom: 16px;
}

.admin-warning strong {
  display: block;
  color: var(--warning);
  font-size: 14px;
  margin-bottom: 6px;
}

.admin-warning p {
  color: var(--text-secondary);
  font-size: 13px;
  margin: 0 0 10px 0;
  line-height: 1.5;
}

.admin-btn {
  margin-top: 4px;
}
</style>
