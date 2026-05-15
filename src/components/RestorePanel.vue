<script setup lang="ts">
import { computed, ref } from "vue";
import { useForm, useField } from "vee-validate";
import { toTypedSchema } from "@vee-validate/zod";
import { z } from "zod";
import { open } from "@tauri-apps/plugin-dialog";
import DiskDialog from "./DiskDialog.vue";
import { startRestore } from "@/composables/useCloneApi";
import { formatSize } from "@/utils/format";
import type { DiskInfo } from "@/types/disk";

const props = defineProps<{
  disks: DiskInfo[];
  isRunning: boolean;
}>();

const emit = defineEmits<{
  (e: "start", promise: Promise<void>): void;
}>();

const showDialog = ref(false);

const schema = toTypedSchema(
  z.object({
    imagePath: z.string().min(1, "Please select an image file"),
    targetDisk: z.string().min(1, "Please select a target disk"),
  })
);

const { errors, handleSubmit, resetForm } = useForm({
  validationSchema: schema,
  initialValues: { imagePath: "", targetDisk: "" },
});

const { value: imagePath } = useField<string>("imagePath");
const { value: targetDisk } = useField<string>("targetDisk");

const selectedDisk = computed(() =>
  props.disks.find((d) => d.DeviceID === targetDisk.value)
);

async function browseImage() {
  const path = await open({
    multiple: false,
    filters: [{ name: "Disk Image", extensions: ["img", "raw", "dd", "*"] }],
  });
  if (path && typeof path === "string") {
    imagePath.value = path;
  }
}

const onSubmit = handleSubmit(async (values) => {
  emit(
    "start",
    startRestore({
      imagePath: values.imagePath,
      targetPath: values.targetDisk,
    })
  );
});

function reset() {
  resetForm();
}

defineExpose({ reset });
</script>

<template>
  <div>
    <div class="section">
      <h2>Source Image</h2>
      <div class="path-row">
        <input
          type="text"
          v-model="imagePath"
          placeholder="Path to the disk image file..."
          :disabled="isRunning"
          class="path-input"
        />
        <button class="btn btn-secondary" @click="browseImage" :disabled="isRunning">Browse</button>
      </div>
      <p v-if="errors.imagePath" class="field-error">{{ errors.imagePath }}</p>
    </div>

    <div class="section">
      <h2>Target Disk</h2>
      <div class="warning-box">
        WARNING: All data on the selected disk will be PERMANENTLY destroyed!
      </div>
      <div v-if="selectedDisk" class="selected-disk">
        <div class="selected-info">
          <span class="selected-name">{{ selectedDisk.Caption }}</span>
          <span v-if="selectedDisk.DriveLetters" class="selected-letters">
            ({{ selectedDisk.DriveLetters }})
          </span>
          <span class="selected-size">{{ formatSize(selectedDisk.Size) }}</span>
        </div>
        <button class="btn btn-secondary" @click="showDialog = true" :disabled="isRunning">
          Change
        </button>
      </div>
      <button v-else class="btn btn-secondary select-btn" @click="showDialog = true" :disabled="isRunning">
        Select Target Disk
      </button>
      <p v-if="errors.targetDisk" class="field-error">{{ errors.targetDisk }}</p>
    </div>

    <DiskDialog
      v-model:show="showDialog"
      v-model="targetDisk"
      :disks="disks"
      title="Select Target Disk"
    />

    <div class="actions">
      <button class="btn btn-primary" @click="onSubmit" :disabled="isRunning">
        Start Restore
      </button>
    </div>
  </div>
</template>

<style scoped>
.select-btn {
  width: 100%;
  margin-top: 4px;
}

.selected-disk {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 14px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
}

.selected-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.selected-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.selected-letters {
  font-size: 13px;
  color: var(--accent);
  font-weight: 500;
}

.selected-size {
  font-size: 12px;
  color: var(--text-secondary);
}
</style>
