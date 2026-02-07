<script setup lang="ts">
import { ref, onMounted } from 'vue';

const props = defineProps<{
  message: string;
  type: 'error' | 'success' | 'info';
  duration?: number;
}>();

const emit = defineEmits(['close']);

const isVisible = ref(true);

onMounted(() => {
  setTimeout(() => {
    isVisible.value = false;
    setTimeout(() => emit('close'), 500); // Wait for transition
  }, props.duration || 3000);
});
</script>

<template>
  <transition name="pop">
    <div v-if="isVisible" :class="['notification', type]">
      <div class="line top"></div>
      <div class="content">
          <span v-if="type === 'error'" class="icon">⚠️</span>
          <span v-else-if="type === 'success'" class="icon">✔️</span>
          <span v-else class="icon">ℹ️</span>
          <p>{{ message }}</p>
      </div>
      <div class="line bottom"></div>
    </div>
  </transition>
</template>

<style scoped>
.notification {
  position: fixed;
  top: 15px;
  right: 15px;
  background: rgba(26, 36, 51, 0.95);
  backdrop-filter: blur(5px);
  color: white;
  padding: 12px 20px;
  border: 1px solid #c0c0c0;
  box-shadow: 0 10px 20px rgba(0,0,0,0.5);
  z-index: 1000;
  min-width: 180px;
  max-width: 300px;
  font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
  text-transform: uppercase;
  letter-spacing: 1px;
  border-radius: 4px;
}

.line {
    height: 1px;
    background: rgba(192, 192, 192, 0.3);
    width: 30%;
    margin: 0 auto;
}

.top { margin-bottom: 8px; }
.bottom { margin-top: 8px; }

.content {
    display: flex;
    align-items: center;
    gap: 12px;
}

.icon {
    font-size: 1.1rem;
    color: #00f2ff;
}

.error { border-color: #ff4d4d; }
.error .icon { color: #ff4d4d; }

.success { border-color: #00f2ff; }
.success .icon { color: #00f2ff; }

.info { border-color: #0072BB; }
.info .icon { color: #0072BB; }

p {
    margin: 0;
    font-weight: 600;
    font-size: 0.75rem;
    opacity: 0.9;
}

/* Transitions */
.pop-enter-active {
  animation: pop-in 0.3s ease-out;
}

.pop-leave-active {
  animation: pop-out 0.25s ease-in forwards;
}

@keyframes pop-in {
  0% { transform: translateY(-20px) scale(0.9); opacity: 0; }
  100% { transform: translateY(0) scale(1); opacity: 1; }
}

@keyframes pop-out {
  0% { transform: translateY(0) scale(1); opacity: 1; }
  100% { transform: translateY(-10px) scale(0.9); opacity: 0; }
}
</style>


