<script setup lang="ts">
import { computed, ref } from "vue";
import { useForm, useField } from "vee-validate";
import { toTypedSchema } from "@vee-validate/zod";
import { z } from "zod";
import DiskDialog from "./DiskDialog.vue";
import { startDiskToDisk } from "@/composables/useCloneApi";
import { formatSize } from "@/utils/format";
import type { DiskInfo } from "@/types/disk";

const props = defineProps<{
  disks: DiskInfo[];
  isRunning: boolean;
}>();

const emit = defineEmits<{
  (e: "start", promise: Promise<void>): void;
}>();

const showSourceDialog = ref(false);
const showTargetDialog = ref(false);

const schema = toTypedSchema(
  z.object({
    sourceDisk: z.string().min(1, "Please select a source disk"),
    targetDisk: z.string().min(1, "Please select a target disk"),
    verify: z.boolean().default(true),
  }).refine((data) => data.sourceDisk !== data.targetDisk, {
    message: "Source and target must be different disks",
    path: ["targetDisk"],
  })
);

const { errors, handleSubmit, resetForm } = useForm({
  validationSchema: schema,
  initialValues: { sourceDisk: "", targetDisk: "", verify: true },
});

const { value: sourceDisk } = useField<string>("sourceDisk");
const { value: targetDisk } = useField<string>("targetDisk");
const { value: verify } = useField<boolean>("verify");

const selectedSource = computed(() =>
  props.disks.find((d) => d.DeviceID === sourceDisk.value)
);
const selectedTarget = computed(() =>
  props.disks.find((d) => d.DeviceID === targetDisk.value)
);

const isSameDisk = computed(() => {
  return sourceDisk.value && targetDisk.value && sourceDisk.value === targetDisk.value;
});

const onSubmit = handleSubmit(async (values) => {
  const disk = props.disks.find((d) => d.DeviceID === values.sourceDisk);
  if (!disk) return;

  emit(
    "start",
    startDiskToDisk({
      sourcePath: values.sourceDisk,
      targetPath: values.targetDisk,
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
      <div v-if="selectedSource" class="selected-disk">
        <div class="selected-info">
          <span class="selected-name">{{ selectedSource.Caption }}</span>
          <span v-if="selectedSource.DriveLetters" class="selected-letters">
            ({{ selectedSource.DriveLetters }})
          </span>
          <span class="selected-size">{{ formatSize(selectedSource.Size) }}</span>
        </div>
        <button class="btn btn-secondary" @click="showSourceDialog = true" :disabled="isRunning">
          Change
        </button>
      </div>
      <button v-else class="btn btn-secondary select-btn" @click="showSourceDialog = true" :disabled="isRunning">
        Select Source Disk
      </button>
      <p v-if="errors.sourceDisk" class="field-error">{{ errors.sourceDisk }}</p>
    </div>

    <DiskDialog
      v-model:show="showSourceDialog"
      v-model="sourceDisk"
      :disks="disks"
      title="Select Source Disk"
    />

    <div class="section">
      <h2>Target Disk</h2>
      <div class="warning-box">
        WARNING: All data on the selected target disk will be PERMANENTLY destroyed!
      </div>
      <div v-if="isSameDisk" class="error">Source and target must be different disks!</div>
      <div v-if="selectedTarget" class="selected-disk">
        <div class="selected-info">
          <span class="selected-name">{{ selectedTarget.Caption }}</span>
          <span v-if="selectedTarget.DriveLetters" class="selected-letters">
            ({{ selectedTarget.DriveLetters }})
          </span>
          <span class="selected-size">{{ formatSize(selectedTarget.Size) }}</span>
        </div>
        <button class="btn btn-secondary" @click="showTargetDialog = true" :disabled="isRunning">
          Change
        </button>
      </div>
      <button v-else class="btn btn-secondary select-btn" @click="showTargetDialog = true" :disabled="isRunning">
        Select Target Disk
      </button>
      <p v-if="errors.targetDisk" class="field-error">{{ errors.targetDisk }}</p>
    </div>

    <DiskDialog
      v-model:show="showTargetDialog"
      v-model="targetDisk"
      :disks="disks"
      title="Select Target Disk"
    />

    <div class="section">
      <label class="checkbox-label">
        <input type="checkbox" v-model="verify" :disabled="isRunning" />
        Verify with SHA-256 after copy
      </label>
    </div>

    <div class="actions">
      <button class="btn btn-primary" @click="onSubmit" :disabled="isRunning">
        Start Copy
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
