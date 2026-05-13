<script setup lang="ts">
import { useForm, useField } from "vee-validate";
import { toTypedSchema } from "@vee-validate/zod";
import { z } from "zod";
import { save } from "@tauri-apps/plugin-dialog";
import DiskSelector from "./DiskSelector.vue";
import { startClone } from "@/composables/useCloneApi";
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
      <DiskSelector
        :disks="disks"
        v-model="sourceDisk"
        :disabled="isRunning"
        name="clone-source"
      />
      <p v-if="errors.sourceDisk" class="field-error">{{ errors.sourceDisk }}</p>
    </div>

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
