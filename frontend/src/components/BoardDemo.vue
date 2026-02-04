<script setup lang="ts">
import GameBoard from './GameBoard.vue'
</script>

<template>
  <div class="demo-container">
    <GameBoard />
  </div>
</template>

<style scoped>
.demo-container {
  width: 100vw;
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #0c1821 0%, #1a2a3a 50%, #0c1821 100%);
  overflow: auto;
}
</style>
