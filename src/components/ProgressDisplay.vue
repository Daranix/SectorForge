<script setup lang="ts">
import type { ProgressPayload } from "@/types/disk";
import { formatSize, formatSpeed, formatEta, calcPercent } from "@/utils/format";
import { computed } from "vue";

const props = defineProps<{
  progress: ProgressPayload | null;
}>();

const percent = computed(() => {
  if (!props.progress) return "0.0";
  return calcPercent(props.progress.bytes_processed, props.progress.total_bytes);
});
</script>

<template>
  <div class="progress-section">
    <h2>Progress</h2>
    <div class="progress-bar-container">
      <div class="progress-bar" :style="{ width: percent + '%' }"></div>
    </div>
    <div class="progress-stats">
      <span>{{ percent }}%</span>
      <span v-if="progress">
        {{ formatSize(progress.bytes_processed) }} / {{ formatSize(progress.total_bytes) }}
      </span>
      <span v-if="progress && progress.speed_bytes_per_sec > 0">
        {{ formatSpeed(progress.speed_bytes_per_sec) }}
      </span>
      <span v-if="progress && progress.eta_seconds > 0">
        ETA: {{ formatEta(progress.eta_seconds) }}
      </span>
    </div>
  </div>
</template>

<style scoped>
.progress-section {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 16px;
}

.progress-section h2 {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 10px;
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
</style>
