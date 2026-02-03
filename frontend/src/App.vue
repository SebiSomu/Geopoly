<script setup lang="ts">
import { ref } from 'vue'

const mesaj = ref('Salut din Vue 3 + Vite!')

const schimbaMesajul = () => {
  mesaj.value = 'Configurarea funcționează perfect! ✅'
}
</script>

<template>
  <div class="test-container">
    <h1>{{ mesaj }}</h1>
    <button @click="schimbaMesajul">Apasă-mă pentru test</button>
  </div>
</template>

<style scoped>
.test-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100vh;
  font-family: sans-serif;
  background-color: #f4f4f4;
}

button {
  padding: 10px 20px;
  font-size: 1rem;
  cursor: pointer;
  background-color: #42b883;
  color: white;
  border: none;
  border-radius: 4px;
}

button:hover {
  background-color: #35495e;
}
</style>