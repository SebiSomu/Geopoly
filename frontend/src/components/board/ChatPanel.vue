<script setup lang="ts">
import { ref, watch, nextTick } from 'vue';

const props = defineProps<{
  messages: Array<{ sender: string; content: string; timestamp: string }>;
  currentUser: string;
}>();

const emit = defineEmits(['send']);

const newMessage = ref('');
const messagesContainer = ref<HTMLElement | null>(null);

const sendMessage = () => {
  if (newMessage.value.trim()) {
    emit('send', newMessage.value.trim());
    newMessage.value = '';
  }
};

const formatTime = (isoString: string) => {
  try {
    const date = new Date(isoString);
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  } catch (e) {
    return '';
  }
};

watch(() => props.messages, () => {
  nextTick(() => {
    if (messagesContainer.value) {
      messagesContainer.value.scrollTo({
        top: messagesContainer.value.scrollHeight,
        behavior: 'smooth'
      });
    }
  });
}, { deep: true });
</script>

<template>
  <div class="chat-panel">
    <div class="chat-header">
      <h3>Live Chat</h3>
    </div>
    <div class="messages-container" ref="messagesContainer">
      <div 
        v-for="(msg, idx) in messages" 
        :key="idx" 
        class="message-bubble"
        :class="{ 'is-me': msg.sender === currentUser }"
      >
        <div class="message-info">
          <span class="sender">{{ msg.sender }}</span>
          <span class="time">{{ formatTime(msg.timestamp) }}</span>
        </div>
        <div class="message-content">
          {{ msg.content }}
        </div>
      </div>
    </div>
    <div class="chat-input-area">
      <input 
        v-model="newMessage" 
        type="text" 
        placeholder="Type a message..." 
        @keyup.enter="sendMessage"
      />
      <button @click="sendMessage">SEND</button>
    </div>
  </div>
</template>

<style scoped>
.chat-panel {
  position: absolute;
  bottom: 80px;
  right: 20px;
  width: 300px;
  height: 400px;
  background: rgba(12, 24, 40, 0.98);
  border: 1px solid rgba(255, 215, 0, 0.3);
  border-radius: 16px;
  display: flex;
  flex-direction: column;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.8);
  z-index: 101;
  overflow: hidden;
  animation: slide-up 0.3s ease-out;
}

@keyframes slide-up {
  from { opacity: 0; transform: translateY(20px); }
  to { opacity: 1; transform: translateY(0); }
}

.chat-header {
  padding: 12px;
  background: rgba(30, 41, 59, 0.8);
  border-bottom: 1px solid rgba(255, 215, 0, 0.2);
}

.chat-header h3 {
  margin: 0;
  font-size: 0.9rem;
  color: #fbbf24;
  text-transform: uppercase;
  letter-spacing: 1px;
}

.messages-container {
  flex-grow: 1;
  overflow-y: auto;
  padding: 15px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.message-bubble {
  max-width: 85%;
  align-self: flex-start;
}

.message-bubble.is-me {
  align-self: flex-end;
}

.message-info {
  display: flex;
  gap: 8px;
  margin-bottom: 4px;
  font-size: 0.65rem;
}

.sender {
  color: #fbbf24;
  font-weight: 700;
}

.time {
  color: rgba(255, 255, 255, 0.4);
}

.message-content {
  background: rgba(255, 255, 255, 0.05);
  padding: 8px 12px;
  border-radius: 12px;
  border-top-left-radius: 2px;
  font-size: 0.8rem;
  color: #e2e8f0;
  line-height: 1.4;
}

.is-me .message-content {
  background: rgba(251, 191, 36, 0.15);
  border-radius: 12px;
  border-top-right-radius: 2px;
  border-top-left-radius: 12px;
}

.is-me .message-info {
  flex-direction: row-reverse;
}

.chat-input-area {
  padding: 12px;
  background: rgba(30, 41, 59, 0.8);
  display: flex;
  gap: 8px;
}

.chat-input-area input {
  flex-grow: 1;
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  padding: 8px 12px;
  color: white;
  font-size: 0.8rem;
}

.chat-input-area button {
  background: #fbbf24;
  color: #0f172a;
  border: none;
  border-radius: 8px;
  padding: 0 12px;
  font-size: 0.7rem;
  font-weight: 900;
  cursor: pointer;
  transition: all 0.2s;
}

.chat-input-area button:hover {
  background: #f59e0b;
  transform: scale(1.05);
}
</style>
