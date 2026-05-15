<script setup lang="ts">
import type { OperationMode } from "@/types/disk";

const props = defineProps<{
  modelValue: OperationMode;
  disabled?: boolean;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", mode: OperationMode): void;
}>();

const tabs: { key: OperationMode; label: string }[] = [
  { key: "clone", label: "Clone to Image" },
  { key: "restore", label: "Restore from Image" },
  { key: "disk2disk", label: "Disk to Disk" },
  { key: "verify", label: "Verify Image" },
];

function selectTab(mode: OperationMode) {
  if (props.disabled) return;
  emit("update:modelValue", mode);
}
</script>

<template>
  <div class="tabs">
    <button
      v-for="tab in tabs"
      :key="tab.key"
      :class="['tab', modelValue === tab.key ? 'tab-active' : '', disabled ? 'tab-disabled' : '']"
      @click="selectTab(tab.key)"
      :disabled="disabled"
    >
      {{ tab.label }}
    </button>
  </div>
</template>

<style scoped>
.tabs {
  display: flex;
  gap: 2px;
  padding: 12px 24px 0;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
}

.tab {
  padding: 10px 16px;
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

.tab-disabled {
  opacity: 0.5;
  cursor: not-allowed;
  pointer-events: none;
}
</style>
