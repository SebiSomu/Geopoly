<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  type: 'seal' | 'capybara' | 'cat' | 'dog' | string
  selected?: boolean
  taken?: boolean
}

const props = defineProps<Props>()
const emit = defineEmits(['select'])

const tokenData = computed(() => {
  switch (props.type) {
    case 'seal': return { emoji: null, color: '#E0E0E0' }
    case 'cat': return { emoji: '🐱', color: '#FFB74D' }
    case 'dog': return { emoji: '🐶', color: '#B08D57' }
    case 'capybara': return { emoji: null, color: '#8D6E63' }
    default: return { emoji: '❓', color: '#999' }
  }
})

const handleClick = () => {
  if (!props.taken) {
    emit('select', props.type)
  }
}
</script>

<template>
  <div 
    class="character-option" 
    :class="{ 'selected': selected, 'taken': taken }"
    @click="handleClick"
    :style="{ '--token-color': tokenData.color }"
  >
    <div class="token-visual">
      <template v-if="tokenData.emoji">
        <span class="emoji-face">{{ tokenData.emoji }}</span>
      </template>
      <template v-else-if="type === 'seal'">
        <!-- Simple Seal Face SVG -->
        <svg viewBox="0 0 24 24" class="token-svg">
          <circle cx="12" cy="13" r="8" fill="#E0E0E0" />
          <circle cx="9" cy="12" r="1.2" fill="#333" />
          <circle cx="15" cy="12" r="1.2" fill="#333" />
          <ellipse cx="12" cy="16" rx="3.5" ry="2.5" fill="#F5F5F5" />
          <circle cx="12" cy="15.5" r="1" fill="#333" />
          <!-- Whiskers -->
          <path d="M8 16L5 15.5M8 16.5L5 17.5" stroke="#999" stroke-width="0.5" />
          <path d="M16 16L19 15.5M16 16.5L19 17.5" stroke="#999" stroke-width="0.5" />
        </svg>
      </template>
      <template v-else-if="type === 'capybara'">
        <!-- Simple Capybara Face SVG -->
        <svg viewBox="0 0 24 24" class="token-svg">
          <path d="M4 14C4 10 7 7 12 7S20 10 20 14C20 18 17 21 12 21S4 18 4 14Z" fill="#A1887F" />
          <path d="M11 7L7 5L9 8" fill="#A1887F" />
          <path d="M13 7L17 5L15 8" fill="#A1887F" />
          <rect x="8" y="12" width="2" height="1.5" rx="0.5" fill="#333" />
          <rect x="14" y="12" width="2" height="1.5" rx="0.5" fill="#333" />
          <ellipse cx="12" cy="17" rx="4" ry="3" fill="#8D6E63" />
          <circle cx="10.5" cy="16" r="1" fill="#333" />
          <circle cx="13.5" cy="16" r="1" fill="#333" />
        </svg>
      </template>
    </div>
    
    <div v-if="selected" class="selection-indicator"></div>
  </div>
</template>

<style scoped>
.character-option {
  position: relative;
  width: 80px;
  height: 80px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275);
  border: 2px solid transparent;
}

.character-option:hover:not(.taken) {
  background: rgba(255, 255, 255, 0.15);
  transform: translateY(-5px);
}

.character-option.selected {
  background: rgba(138, 43, 226, 0.2);
  border-color: #8a2be2;
  box-shadow: 0 0 15px rgba(138, 43, 226, 0.4);
}

.character-option.taken {
  opacity: 0.5;
  cursor: not-allowed;
  filter: grayscale(1);
}

.token-visual {
  width: 50px;
  height: 50px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.token-svg {
  width: 100%;
  height: 100%;
  filter: drop-shadow(0 2px 4px rgba(0,0,0,0.3));
}

.emoji-face {
  font-size: 3rem;
  line-height: 1;
  filter: drop-shadow(0 2px 4px rgba(0,0,0,0.3));
}

.selection-indicator {
  position: absolute;
  bottom: -10px;
  width: 6px;
  height: 6px;
  background: #8a2be2;
  border-radius: 50%;
  box-shadow: 0 0 5px #8a2be2;
}
</style>
