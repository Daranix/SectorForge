<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { save, open } from "@tauri-apps/plugin-dialog";

interface DiskInfo {
  DeviceID: string;
  Caption: string;
  Size: number;
  MediaType: string | null;
  Partitions: number;
}

interface ProgressEvent {
  bytes_processed: number;
  total_bytes: number;
  speed_bytes_per_sec: number;
  eta_seconds: number;
  percent: number;
}

type OperationMode = "clone" | "restore" | "verify";

const mode = ref<OperationMode>("clone");
const disks = ref<DiskInfo[]>([]);
const loadingDisks = ref(false);
const diskError = ref("");

const selectedSourceDisk = ref("");
const selectedTargetDisk = ref("");
const outputImagePath = ref("");
const inputImagePath = ref("");

const isRunning = ref(false);
const progress = ref<ProgressEvent | null>(null);
const operationError = ref("");
const operationSuccess = ref("");
const verify = ref(true);
const hashResult = ref("");

const formattedSize = (bytes: number): string => {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(2)} ${units[i]}`;
};

const formatSpeed = (bytesPerSec: number): string => {
  return `${formattedSize(bytesPerSec)}/s`;
};

const formatEta = (seconds: number): string => {
  if (seconds <= 0 || !isFinite(seconds)) return "--";
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  if (h > 0) return `${h}h ${m}m ${s}s`;
  if (m > 0) return `${m}m ${s}s`;
  return `${s}s`;
};

const progressPercent = computed(() => {
  if (!progress.value) return 0;
  return Math.min(progress.value.percent, 100).toFixed(1);
});

const canStart = computed(() => {
  if (isRunning.value) return false;
  if (mode.value === "clone") {
    return selectedSourceDisk.value !== "" && outputImagePath.value !== "";
  }
  if (mode.value === "restore") {
    return inputImagePath.value !== "" && selectedTargetDisk.value !== "";
  }
  if (mode.value === "verify") {
    return inputImagePath.value !== "";
  }
  return false;
});

async function loadDisks() {
  loadingDisks.value = true;
  diskError.value = "";
  try {
    const result = await invoke<DiskInfo[]>("list_disks");
    disks.value = result;
  } catch (e) {
    diskError.value = String(e);
    disks.value = [];
  } finally {
    loadingDisks.value = false;
  }
}

async function browseOutputImage() {
  const path = await save({
    defaultPath: "disk_image.img",
    filters: [{ name: "Disk Image", extensions: ["img", "raw", "dd"] }],
  });
  if (path) {
    outputImagePath.value = path;
  }
}

async function browseInputImage() {
  const path = await open({
    multiple: false,
    filters: [{ name: "Disk Image", extensions: ["img", "raw", "dd", "*"] }],
  });
  if (path && typeof path === "string") {
    inputImagePath.value = path;
  }
}

async function startOperation() {
  progress.value = null;
  operationError.value = "";
  operationSuccess.value = "";
  hashResult.value = "";
  isRunning.value = true;

  try {
    if (mode.value === "clone") {
      const disk = disks.value.find((d) => d.DeviceID === selectedSourceDisk.value);
      if (!disk) throw new Error("No disk selected");
      await invoke("start_clone", {
        sourcePath: selectedSourceDisk.value,
        outputPath: outputImagePath.value,
        totalSize: disk.Size,
        verify: verify.value,
      });
      operationSuccess.value = "Clone completed successfully!";
    } else if (mode.value === "restore") {
      await invoke("start_restore", {
        imagePath: inputImagePath.value,
        targetPath: selectedTargetDisk.value,
      });
      operationSuccess.value = "Restore completed successfully!";
    } else if (mode.value === "verify") {
      const hash = await invoke<string>("start_verify", {
        imagePath: inputImagePath.value,
      });
      hashResult.value = hash;
      operationSuccess.value = `SHA-256: ${hash}`;
    }
  } catch (e) {
    operationError.value = String(e);
  } finally {
    isRunning.value = false;
  }
}

async function cancelOperation() {
  try {
    await invoke("cancel_operation");
    operationError.value = "Operation cancelled";
  } catch (e) {
    console.error("Cancel error:", e);
  }
}

function resetOperation() {
  progress.value = null;
  operationError.value = "";
  operationSuccess.value = "";
  hashResult.value = "";
}

loadDisks();

listen<ProgressEvent>("clone-progress", (event) => {
  progress.value = event.payload;
});
listen<ProgressEvent>("restore-progress", (event) => {
  progress.value = event.payload;
});
listen<ProgressEvent>("hash-progress", (event) => {
  progress.value = event.payload;
});
listen<ProgressEvent>("verify-progress", (event) => {
  progress.value = event.payload;
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

    <div class="tabs">
      <button
        :class="['tab', mode === 'clone' ? 'tab-active' : '']"
        @click="mode = 'clone'; resetOperation()"
      >
        Clone to Image
      </button>
      <button
        :class="['tab', mode === 'restore' ? 'tab-active' : '']"
        @click="mode = 'restore'; resetOperation()"
      >
        Restore from Image
      </button>
      <button
        :class="['tab', mode === 'verify' ? 'tab-active' : '']"
        @click="mode = 'verify'; resetOperation()"
      >
        Verify Image
      </button>
    </div>

    <div class="content">
      <!-- CLONE MODE -->
      <template v-if="mode === 'clone'">
        <div class="section">
          <h2>Source Disk</h2>
          <div class="disk-list" v-if="disks.length > 0">
            <label
              v-for="disk in disks"
              :key="disk.DeviceID"
              :class="['disk-item', selectedSourceDisk === disk.DeviceID ? 'disk-selected' : '']"
            >
              <input
                type="radio"
                :value="disk.DeviceID"
                v-model="selectedSourceDisk"
                :disabled="isRunning"
              />
              <div class="disk-info">
                <span class="disk-name">{{ disk.Caption }}</span>
                <span class="disk-details">
                  {{ formattedSize(disk.Size) }} ·
                  {{ disk.MediaType || 'Unknown type' }} ·
                  {{ disk.Partitions }} partition{{ disk.Partitions !== 1 ? 's' : '' }}
                </span>
                <span class="disk-path">{{ disk.DeviceID }}</span>
              </div>
            </label>
          </div>
          <div v-else-if="diskError" class="error">{{ diskError }}</div>
          <div v-else class="empty">No disks found</div>
          <button class="btn btn-secondary" @click="loadDisks" :disabled="loadingDisks">
            {{ loadingDisks ? "Loading..." : "Refresh Disks" }}
          </button>
        </div>

        <div class="section">
          <h2>Output Image</h2>
          <div class="path-row">
            <input
              type="text"
              v-model="outputImagePath"
              placeholder="Path to save the disk image..."
              :disabled="isRunning"
              class="path-input"
            />
            <button class="btn btn-secondary" @click="browseOutputImage" :disabled="isRunning">Browse</button>
          </div>
          <label class="checkbox-label">
            <input type="checkbox" v-model="verify" :disabled="isRunning" />
            Verify with SHA-256 after clone
          </label>
        </div>
      </template>

      <!-- RESTORE MODE -->
      <template v-if="mode === 'restore'">
        <div class="section">
          <h2>Source Image</h2>
          <div class="path-row">
            <input
              type="text"
              v-model="inputImagePath"
              placeholder="Path to the disk image file..."
              :disabled="isRunning"
              class="path-input"
            />
            <button class="btn btn-secondary" @click="browseInputImage" :disabled="isRunning">Browse</button>
          </div>
        </div>

        <div class="section">
          <h2>Target Disk</h2>
          <div class="warning-box">
            WARNING: All data on the selected disk will be PERMANENTLY destroyed!
          </div>
          <div class="disk-list" v-if="disks.length > 0">
            <label
              v-for="disk in disks"
              :key="disk.DeviceID"
              :class="['disk-item', selectedTargetDisk === disk.DeviceID ? 'disk-selected' : '']"
            >
              <input
                type="radio"
                :value="disk.DeviceID"
                v-model="selectedTargetDisk"
                :disabled="isRunning"
              />
              <div class="disk-info">
                <span class="disk-name">{{ disk.Caption }}</span>
                <span class="disk-details">
                  {{ formattedSize(disk.Size) }} ·
                  {{ disk.MediaType || 'Unknown type' }} ·
                  {{ disk.Partitions }} partition{{ disk.Partitions !== 1 ? 's' : '' }}
                </span>
                <span class="disk-path">{{ disk.DeviceID }}</span>
              </div>
            </label>
          </div>
          <div v-else-if="diskError" class="error">{{ diskError }}</div>
          <div v-else class="empty">No disks found</div>
          <button class="btn btn-secondary" @click="loadDisks" :disabled="loadingDisks">
            {{ loadingDisks ? "Loading..." : "Refresh Disks" }}
          </button>
        </div>
      </template>

      <!-- VERIFY MODE -->
      <template v-if="mode === 'verify'">
        <div class="section">
          <h2>Verify Image</h2>
          <p class="description">Compute the SHA-256 hash of a disk image file.</p>
          <div class="path-row">
            <input
              type="text"
              v-model="inputImagePath"
              placeholder="Path to the disk image file..."
              :disabled="isRunning"
              class="path-input"
            />
            <button class="btn btn-secondary" @click="browseInputImage" :disabled="isRunning">Browse</button>
          </div>
          <div v-if="hashResult" class="hash-result">
            <strong>SHA-256:</strong>
            <code>{{ hashResult }}</code>
          </div>
        </div>
      </template>

      <!-- PROGRESS -->
      <div v-if="isRunning || progress" class="section progress-section">
        <h2>Progress</h2>
        <div class="progress-bar-container">
          <div class="progress-bar" :style="{ width: progressPercent + '%' }"></div>
        </div>
        <div class="progress-stats">
          <span>{{ progressPercent }}%</span>
          <span v-if="progress">
            {{ formattedSize(progress.bytes_processed) }} / {{ formattedSize(progress.total_bytes) }}
          </span>
          <span v-if="progress && progress.speed_bytes_per_sec > 0">
            {{ formatSpeed(progress.speed_bytes_per_sec) }}
          </span>
          <span v-if="progress && progress.eta_seconds > 0">
            ETA: {{ formatEta(progress.eta_seconds) }}
          </span>
        </div>
      </div>

      <!-- RESULT MESSAGES -->
      <div v-if="operationSuccess && !isRunning" class="success">{{ operationSuccess }}</div>
      <div v-if="operationError && !isRunning" class="error">{{ operationError }}</div>

      <!-- ACTION BUTTONS -->
      <div class="actions">
        <button
          class="btn btn-primary"
          @click="startOperation"
          :disabled="!canStart"
          v-if="!isRunning"
        >
          {{ mode === 'clone' ? 'Start Clone' : mode === 'restore' ? 'Start Restore' : 'Verify Image' }}
        </button>
        <button
          class="btn btn-danger"
          @click="cancelOperation"
          v-if="isRunning"
        >
          Cancel
        </button>
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

.tabs {
  display: flex;
  gap: 2px;
  padding: 12px 24px 0;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
}

.tab {
  padding: 10px 20px;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  transition: all 0.2s;
  font-family: inherit;
}

.tab:hover {
  color: var(--text-primary);
  background: var(--bg-tertiary);
}

.tab-active {
  color: var(--accent);
  border-bottom-color: var(--accent);
  background: var(--accent-bg);
}

.content {
  flex: 1;
  padding: 20px 24px;
  overflow-y: auto;
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

.progress-section {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 16px;
}

.progress-bar-container {
  width: 100%;
  height: 8px;
  background: var(--bg-tertiary);
  border-radius: 4px;
  overflow: hidden;
  margin-bottom: 10px;
}

.progress-bar {
  height: 100%;
  background: linear-gradient(90deg, var(--accent), var(--accent-hover));
  border-radius: 4px;
  transition: width 0.3s ease;
}

.progress-stats {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  color: var(--text-secondary);
  gap: 16px;
  flex-wrap: wrap;
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
</style>