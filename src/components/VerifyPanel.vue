<script setup lang="ts">
import { useForm, useField } from "vee-validate";
import { toTypedSchema } from "@vee-validate/zod";
import { z } from "zod";
import { open } from "@tauri-apps/plugin-dialog";
import { startVerify } from "@/composables/useCloneApi";

defineProps<{
  isRunning: boolean;
}>();

const emit = defineEmits<{
  (e: "hash", hash: string): void;
  (e: "start", promise: Promise<string>): void;
}>();

const schema = toTypedSchema(
  z.object({
    imagePath: z.string().min(1, "Please select an image file"),
  })
);

const { errors, handleSubmit, resetForm } = useForm({
  validationSchema: schema,
  initialValues: { imagePath: "" },
});

const { value: imagePath } = useField<string>("imagePath");

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
  const promise = startVerify(values.imagePath);
  emit("start", promise);
  const hash = await promise;
  emit("hash", hash);
});

function reset() {
  resetForm();
}

defineExpose({ reset });
</script>

<template>
  <div>
    <div class="section">
      <h2>Verify Image</h2>
      <p class="description">Compute the SHA-256 hash of a disk image file.</p>
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

    <div class="actions">
      <button class="btn btn-primary" @click="onSubmit" :disabled="isRunning">
        Verify Image
      </button>
    </div>
  </div>
</template>
