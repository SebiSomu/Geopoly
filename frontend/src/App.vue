<script setup lang="ts">
import { ref } from 'vue';
import GameBoard from './components/board/GameBoard.vue';
import StartScreen from './components/login/StartScreen.vue';

const gameStarted = ref(false);

const startGame = () => {
  gameStarted.value = true;
};
</script>

<template>
  <StartScreen v-if="!gameStarted" @start-game="startGame" />
  <GameBoard v-else />
</template>

<style>
body {
  margin: 0;
  padding: 0;
  overflow: hidden;
  background-color: black; /* Ensure background matches when components load */
}
</style>