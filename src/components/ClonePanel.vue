<script setup lang="ts">
import { computed, ref } from "vue";
import { useForm, useField } from "vee-validate";
import { toTypedSchema } from "@vee-validate/zod";
import { z } from "zod";
import { save } from "@tauri-apps/plugin-dialog";
import DiskDialog from "./DiskDialog.vue";
import { startClone } from "@/composables/useCloneApi";
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
    sourceDisk: z.string().min(1, "Please select a source disk"),
    outputPath: z.string().min(1, "Please select an output path"),
    verify: z.boolean().default(true),
  })
);

const { errors, handleSubmit, resetForm } = useForm({
  validationSchema: schema,
  initialValues: { sourceDisk: "", outputPath: "", verify: true },
});

const { value: sourceDisk } = useField<string>("sourceDisk");
const { value: outputPath } = useField<string>("outputPath");
const { value: verify } = useField<boolean>("verify");

const selectedDisk = computed(() =>
  props.disks.find((d) => d.DeviceID === sourceDisk.value)
);

async function browseOutput() {
  const path = await save({
    defaultPath: "disk_image.img",
    filters: [{ name: "Disk Image", extensions: ["img", "raw", "dd"] }],
  });
  if (path) {
    outputPath.value = path;
  }
}

const onSubmit = handleSubmit(async (values) => {
  const disk = props.disks.find((d) => d.DeviceID === values.sourceDisk);
  if (!disk) return;

  emit(
    "start",
    startClone({
      sourcePath: values.sourceDisk,
      outputPath: values.outputPath,
      totalSize: disk.Size,
      verify: values.verify,
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
      <h2>Source Disk</h2>
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
        Select Disk
      </button>
      <p v-if="errors.sourceDisk" class="field-error">{{ errors.sourceDisk }}</p>
    </div>

    <DiskDialog
      v-model:show="showDialog"
      v-model="sourceDisk"
      :disks="disks"
      title="Select Source Disk"
    />

    <div class="section">
      <h2>Output Image</h2>
      <div class="path-row">
        <input
          type="text"
          v-model="outputPath"
          placeholder="Path to save the disk image..."
          :disabled="isRunning"
          class="path-input"
        />
        <button class="btn btn-secondary" @click="browseOutput" :disabled="isRunning">Browse</button>
      </div>
      <p v-if="errors.outputPath" class="field-error">{{ errors.outputPath }}</p>
      <label class="checkbox-label">
        <input type="checkbox" v-model="verify" :disabled="isRunning" />
        Verify with SHA-256 after clone
      </label>
    </div>

    <div class="actions">
      <button class="btn btn-primary" @click="onSubmit" :disabled="isRunning">
        Start Clone
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
