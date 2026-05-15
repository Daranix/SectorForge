<script setup lang="ts">
import { ref, watch } from "vue";
import DiskSelector from "./DiskSelector.vue";
import type { DiskInfo } from "@/types/disk";

const props = defineProps<{
  show: boolean;
  disks: DiskInfo[];
  title: string;
  modelValue: string;
}>();

const emit = defineEmits<{
  (e: "update:show", value: boolean): void;
  (e: "update:modelValue", value: string): void;
}>();

const selected = ref(props.modelValue);

watch(
  () => props.show,
  (visible) => {
    if (visible) {
      selected.value = props.modelValue;
    }
  }
);

function close() {
  emit("update:show", false);
}

function confirm() {
  emit("update:modelValue", selected.value);
  close();
}
</script>

<template>
  <Teleport to="body">
    <div v-if="show" class="dialog-overlay" @click.self="close">
      <div class="dialog">
        <div class="dialog-header">
          <h2>{{ title }}</h2>
          <button class="dialog-close" @click="close" aria-label="Close">&times;</button>
        </div>
        <div class="dialog-body">
          <DiskSelector
            :disks="disks"
            v-model="selected"
            name="dialog"
          />
        </div>
        <div class="dialog-footer">
          <button class="btn btn-secondary" @click="close">Cancel</button>
          <button class="btn btn-primary" @click="confirm" :disabled="!selected">
            Select
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(2px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.dialog {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  width: 100%;
  max-width: 520px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.4);
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
}

.dialog-header h2 {
  font-size: 15px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
}

.dialog-close {
  background: none;
  border: none;
  color: var(--text-secondary);
  font-size: 22px;
  cursor: pointer;
  line-height: 1;
  padding: 0 4px;
}

.dialog-close:hover {
  color: var(--text-primary);
}

.dialog-body {
  padding: 16px 20px;
  overflow-y: auto;
  flex: 1;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding: 14px 20px;
  border-top: 1px solid var(--border);
}
</style>
