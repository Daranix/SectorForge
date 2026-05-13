<script setup lang="ts">
import { computed } from "vue";
import { useForm, useField } from "vee-validate";
import { toTypedSchema } from "@vee-validate/zod";
import { z } from "zod";
import DiskSelector from "./DiskSelector.vue";
import { startDiskToDisk } from "@/composables/useCloneApi";
import type { DiskInfo } from "@/types/disk";

const props = defineProps<{
  disks: DiskInfo[];
  isRunning: boolean;
}>();

const emit = defineEmits<{
  (e: "start", promise: Promise<void>): void;
}>();

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
      <DiskSelector
        :disks="disks"
        v-model="sourceDisk"
        :disabled="isRunning"
        name="d2d-source"
      />
      <p v-if="errors.sourceDisk" class="field-error">{{ errors.sourceDisk }}</p>
    </div>

    <div class="section">
      <h2>Target Disk</h2>
      <div class="warning-box">
        WARNING: All data on the selected target disk will be PERMANENTLY destroyed!
      </div>
      <div v-if="isSameDisk" class="error">Source and target must be different disks!</div>
      <DiskSelector
        :disks="disks"
        v-model="targetDisk"
        :disabled="isRunning"
        name="d2d-target"
      />
      <p v-if="errors.targetDisk" class="field-error">{{ errors.targetDisk }}</p>
    </div>

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
