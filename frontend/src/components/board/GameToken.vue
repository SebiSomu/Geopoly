<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  type: 'seal' | 'capybara' | 'cat' | 'dog'
  color?: string
  highlight?: boolean
}

const props = defineProps<Props>()

const tokenData = computed(() => {
  switch (props.type) {
    case 'seal': return { emoji: null, color: '#B0BEC5' }
    case 'cat': return { emoji: '🐱', color: '#FFB74D' }
    case 'dog': return { emoji: '🐶', color: '#B08D57' }
    case 'capybara': return { emoji: null, color: '#8D6E63' }
    default: return { emoji: '❓', color: '#999' }
  }
})
</script>

<template>
  <div 
    class="game-token" 
    :class="{ highlighted: highlight }"
    :style="{ '--token-color': color || tokenData.color }"
  >
    <div class="token-base">
      <div class="token-pedestal"></div>
      <div class="token-icon">
        <template v-if="tokenData.emoji">
          <span class="emoji-face">{{ tokenData.emoji }}</span>
        </template>
        <template v-else-if="type === 'seal'">
          <!-- Enlarged Seal Face SVG -->
          <svg viewBox="0 0 24 24" class="token-svg">
            <circle cx="12" cy="12" r="10.5" fill="#E0E0E0" />
            <circle cx="8.5" cy="11.5" r="1.5" fill="#333" />
            <circle cx="15.5" cy="11.5" r="1.5" fill="#333" />
            <ellipse cx="12" cy="16.5" rx="4.5" ry="3" fill="#F5F5F5" />
            <circle cx="12" cy="16" r="1.2" fill="#333" />
            <path d="M7.5 16.5L4 16M7.5 17L4 18.5" stroke="#999" stroke-width="0.6" />
            <path d="M16.5 16.5L20 16M16.5 17L20 18.5" stroke="#999" stroke-width="0.6" />
          </svg>
        </template>
        <template v-else-if="type === 'capybara'">
          <!-- Enlarged Capybara Face SVG -->
          <svg viewBox="0 0 24 24" class="token-svg">
            <path d="M2 13C2 8 6 5 12 5S22 8 22 13C22 18 18 21 12 21S2 18 2 13Z" fill="#A1887F" />
            <path d="M10.5 5.5L5 3.5L7 7" fill="#A1887F" />
            <path d="M13.5 5.5L19 3.5L17 7" fill="#A1887F" />
            <rect x="7" y="11" width="3" height="2" rx="0.5" fill="#333" />
            <rect x="14" y="11" width="3" height="2" rx="0.5" fill="#333" />
            <ellipse cx="12" cy="17" rx="5" ry="4" fill="#8D6E63" />
            <circle cx="10.5" cy="16" r="1.2" fill="#333" />
            <circle cx="13.5" cy="16" r="1.2" fill="#333" />
          </svg>
        </template>
      </div>
    </div>
    <div class="token-shadow"></div>
  </div>
</template>

<style scoped>
.game-token {
  position: relative;
  width: var(--token-size, 32px);
  height: var(--token-size, 32px);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.3s ease;
  z-index: 20;
}

.game-token.highlighted {
  filter: drop-shadow(0 0 12px #FFD700);
  animation: pulse-gold 1.5s infinite ease-in-out;
}

@keyframes pulse-gold {
  0% { 
    transform: scale(1); 
    filter: drop-shadow(0 0 5px #FFD700) brightness(1); 
  }
  50% { 
    transform: scale(1.25); 
    filter: drop-shadow(0 0 25px #FFD700) brightness(1.3); 
  }
  100% { 
    transform: scale(1); 
    filter: drop-shadow(0 0 5px #FFD700) brightness(1); 
  }
}

.token-base {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.token-icon {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2;
}

.emoji-face {
  font-size: 1.8rem;
  line-height: 1;
}

.token-svg {
  width: 100%;
  height: 100%;
  padding: 1px;
}

.token-pedestal {
  display: none; /* Removed as per user request for face-only design */
}

.token-shadow {
  display: none; /* Removed as per user request for face-only design */
}
</style>
