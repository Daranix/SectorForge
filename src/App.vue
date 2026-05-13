<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
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
});

onUnmounted(() => {
  cleanup();
});
</script>

<template>
  <div class="app">
    <header class="header">
      <div class="logo">
        <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="2" y="2" width="20" height="20" rx="2"/>
          <circle cx="12" cy="12" r="4"/>
          <circle cx="12" cy="12" r="1"/>
        </svg>
        <h1>CloneTool</h1>
      </div>
      <span class="subtitle">Disk Cloner</span>
    </header>

    <TabBar v-model="mode" />

    <div class="content">
      <div v-if="showAdminWarning" class="admin-warning">
        <strong>Administrator privileges required</strong>
        <p>
          Physical disk access is restricted on this system.
          Please close CloneTool and restart it as Administrator
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
:root {
  --bg-primary: #0f1117;
  --bg-secondary: #1a1d2e;
  --bg-tertiary: #252940;
  --bg-hover: #2d3250;
  --text-primary: #e4e6f0;
  --text-secondary: #8b8fa8;
  --text-muted: #5c6078;
  --accent: #4f6ef7;
  --accent-hover: #6b84f9;
  --accent-bg: rgba(79, 110, 247, 0.12);
  --border: #2d3250;
  --success: #2dd4a0;
  --success-bg: rgba(45, 212, 160, 0.1);
  --danger: #f0465a;
  --danger-bg: rgba(240, 70, 90, 0.1);
  --warning: #f5a623;
  --warning-bg: rgba(245, 166, 35, 0.1);
  --radius: 10px;
  --radius-sm: 6px;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  background: var(--bg-primary);
  color: var(--text-primary);
  overflow-x: hidden;
}

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

.section {
  margin-bottom: 20px;
}

.section h2 {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 10px;
}

.disk-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 10px;
}

.disk-item {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 12px 14px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all 0.15s;
}

.disk-item:hover {
  background: var(--bg-hover);
  border-color: var(--text-muted);
}

.disk-selected {
  background: var(--accent-bg) !important;
  border-color: var(--accent) !important;
}

.disk-conflict {
  border-color: var(--danger) !important;
  background: var(--danger-bg) !important;
}

.disk-item input[type="radio"] {
  margin-top: 3px;
  accent-color: var(--accent);
}

.disk-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.disk-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.drive-letters {
  font-weight: 500;
  color: var(--accent);
  margin-left: 4px;
}

.disk-details {
  font-size: 12px;
  color: var(--text-secondary);
}

.disk-path {
  font-size: 11px;
  font-family: 'Cascadia Code', 'Fira Code', monospace;
  color: var(--text-muted);
}

.path-row {
  display: flex;
  gap: 8px;
  margin-bottom: 10px;
}

.path-input {
  flex: 1;
  padding: 9px 12px;
  font-size: 13px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-family: 'Cascadia Code', 'Fira Code', monospace;
  outline: none;
  transition: border-color 0.2s;
}

.path-input:focus {
  border-color: var(--accent);
}

.path-input::placeholder {
  color: var(--text-muted);
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--text-secondary);
  cursor: pointer;
  margin-top: 8px;
}

.checkbox-label input[type="checkbox"] {
  accent-color: var(--accent);
}

.warning-box {
  padding: 10px 14px;
  background: var(--warning-bg);
  border: 1px solid var(--warning);
  border-radius: var(--radius-sm);
  color: var(--warning);
  font-size: 13px;
  font-weight: 600;
  margin-bottom: 10px;
}

.btn {
  padding: 9px 18px;
  font-size: 13px;
  font-weight: 600;
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all 0.15s;
  font-family: inherit;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary {
  background: var(--accent);
  color: #fff;
}

.btn-primary:hover:not(:disabled) {
  background: var(--accent-hover);
}

.btn-secondary {
  background: var(--bg-tertiary);
  color: var(--text-primary);
  border: 1px solid var(--border);
}

.btn-secondary:hover:not(:disabled) {
  background: var(--bg-hover);
}

.btn-danger {
  background: var(--danger);
  color: #fff;
}

.btn-danger:hover:not(:disabled) {
  background: #d9364e;
}

.success {
  padding: 12px 14px;
  background: var(--success-bg);
  border: 1px solid var(--success);
  border-radius: var(--radius-sm);
  color: var(--success);
  font-size: 13px;
  font-weight: 600;
  margin-bottom: 16px;
}

.error {
  padding: 12px 14px;
  background: var(--danger-bg);
  border: 1px solid var(--danger);
  border-radius: var(--radius-sm);
  color: var(--danger);
  font-size: 13px;
  font-weight: 600;
  margin-bottom: 16px;
  word-break: break-word;
}

.empty {
  padding: 20px;
  text-align: center;
  color: var(--text-muted);
  font-size: 13px;
}

.description {
  font-size: 13px;
  color: var(--text-secondary);
  margin-bottom: 12px;
}

.field-error {
  font-size: 12px;
  color: var(--danger);
  margin-top: 4px;
}

.hash-result {
  margin-top: 12px;
  padding: 12px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-sm);
  font-size: 12px;
  word-break: break-all;
}

.hash-result strong {
  color: var(--success);
  display: block;
  margin-bottom: 6px;
}

.hash-result code {
  font-family: 'Cascadia Code', 'Fira Code', monospace;
  color: var(--text-primary);
  font-size: 11px;
}

.actions {
  display: flex;
  gap: 10px;
  margin-top: 8px;
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
