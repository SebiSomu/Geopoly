<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  unreadCount: number;
  isOpen: boolean;
}>();

const emit = defineEmits(['toggle']);

const opacity = computed(() => {
  if (props.isOpen) return 1;
  return props.unreadCount > 0 ? 1 : 0.4;
});

const toggleChat = () => {
  emit('toggle');
};
</script>

<template>
  <div 
    class="chat-icon-container" 
    :class="{ 'has-unread': unreadCount > 0, 'is-open': isOpen }"
    :style="{ opacity: opacity }"
    @click="toggleChat"
  >
    <div class="chat-circle">
      <span class="chat-symbol">💬</span>
      <div v-if="unreadCount > 0" class="unread-badge">
        {{ unreadCount > 9 ? '9+' : unreadCount }}
      </div>
    </div>
  </div>
</template>

<style scoped>
.chat-icon-container {
  position: absolute;
  bottom: 20px;
  right: 20px;
  width: 50px;
  height: 50px;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  z-index: 100;
}

.chat-circle {
  width: 100%;
  height: 100%;
  background: linear-gradient(135deg, #1e293b, #0f172a);
  border: 2px solid rgba(255, 215, 0, 0.3);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
  position: relative;
}

.chat-symbol {
  font-size: 1.5rem;
  filter: drop-shadow(0 0 5px rgba(255, 215, 0, 0.2));
}

.unread-badge {
  position: absolute;
  top: -5px;
  right: -5px;
  background: #ef4444;
  color: white;
  border-radius: 50%;
  width: 20px;
  height: 20px;
  font-size: 0.7rem;
  font-weight: 900;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 2px solid #0f172a;
  animation: pulse 2s infinite;
}

.has-unread .chat-circle {
  border-color: #fbbf24;
  box-shadow: 0 0 15px rgba(251, 191, 36, 0.4);
}

.is-open .chat-circle {
  transform: rotate(90deg);
  background: #fbbf24;
}

.is-open .chat-symbol {
  color: #0f172a;
}

@keyframes pulse {
  0% { transform: scale(1); box-shadow: 0 0 0 0 rgba(239, 68, 68, 0.7); }
  70% { transform: scale(1.1); box-shadow: 0 0 0 10px rgba(239, 68, 68, 0); }
  100% { transform: scale(1); box-shadow: 0 0 0 0 rgba(239, 68, 68, 0); }
}

.chat-icon-container:hover {
  transform: scale(1.1);
  opacity: 1 !important;
}
</style>
