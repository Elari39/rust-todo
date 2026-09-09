<script setup lang="ts">
defineProps<{
  title: string;
  body: string;
}>();
</script>

<template>
  <section class="workspace">
    <div class="placeholder">
      <h2>{{ title }}</h2>
      <p>{{ body }}</p>
    </div>
  </section>
</template>
