<script setup lang="ts">
import { useForm, useField } from "vee-validate";
import { toTypedSchema } from "@vee-validate/zod";
import { z } from "zod";
import { open } from "@tauri-apps/plugin-dialog";
import DiskSelector from "./DiskSelector.vue";
import { startRestore } from "@/composables/useCloneApi";
import type { DiskInfo } from "@/types/disk";

defineProps<{
  disks: DiskInfo[];
  isRunning: boolean;
}>();

const emit = defineEmits<{
  (e: "start", promise: Promise<void>): void;
}>();

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
      <DiskSelector
        :disks="disks"
        v-model="targetDisk"
        :disabled="isRunning"
        name="restore-target"
      />
      <p v-if="errors.targetDisk" class="field-error">{{ errors.targetDisk }}</p>
    </div>

    <div class="actions">
      <button class="btn btn-primary" @click="onSubmit" :disabled="isRunning">
        Start Restore
      </button>
    </div>
  </div>
</template>
