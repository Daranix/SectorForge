<script setup lang="ts">
import type { DiskInfo } from "@/types/disk";
import { formatSize } from "@/utils/format";

defineProps<{
  disks: DiskInfo[];
  modelValue: string;
  disabled?: boolean;
  name?: string;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
}>();

function onChange(deviceId: string) {
  emit("update:modelValue", deviceId);
}
</script>

<template>
  <div class="disk-list" v-if="disks.length > 0">
    <label
      v-for="disk in disks"
      :key="name ? `${name}-${disk.DeviceID}` : disk.DeviceID"
      :class="['disk-item', modelValue === disk.DeviceID ? 'disk-selected' : '']"
    >
      <input
        type="radio"
        :value="disk.DeviceID"
        :checked="modelValue === disk.DeviceID"
        @change="onChange(disk.DeviceID)"
        :disabled="disabled"
        :name="name"
      />
      <div class="disk-info">
        <span class="disk-name">
          {{ disk.Caption }}
          <template v-if="disk.DriveLetters">
            <span class="drive-letters">({{ disk.DriveLetters }})</span>
          </template>
        </span>
        <span class="disk-details">
          {{ formatSize(disk.Size) }} total
          <template v-if="disk.AvailableSpace > 0">
            · {{ formatSize(disk.AvailableSpace) }} free
          </template>
          · {{ disk.MediaType || 'Unknown type' }}
          · {{ disk.Partitions }} partition{{ disk.Partitions !== 1 ? 's' : '' }}
        </span>
        <span class="disk-path">{{ disk.DeviceID }}</span>
      </div>
    </label>
  </div>
  <div v-else class="empty">No disks found</div>
</template>
