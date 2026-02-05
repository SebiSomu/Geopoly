<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  type: 'seal' | 'capybara' | 'cat' | 'dog'
  color?: string
}

const props = defineProps<Props>()

const tokenData = computed(() => {
  switch (props.type) {
    case 'seal': return { emoji: null, color: '#E0E0E0' }
    case 'cat': return { emoji: '🐱', color: '#FFB74D' }
    case 'dog': return { emoji: '🐶', color: '#B08D57' }
    case 'capybara': return { emoji: null, color: '#8D6E63' }
    default: return { emoji: '❓', color: '#999' }
  }
})
</script>

<template>
  <div class="game-token" :style="{ '--token-color': color || tokenData.color }">
    <div class="token-base">
      <div class="token-pedestal"></div>
      <div class="token-icon">
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
    </div>
    <div class="token-shadow"></div>
  </div>
</template>

<style scoped>
.game-token {
  position: relative;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  filter: drop-shadow(0 2px 4px rgba(0,0,0,0.3));
  transition: all 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275);
  cursor: pointer;
  z-index: 20;
}

.game-token:hover {
  transform: translateY(-8px) scale(1.2);
  z-index: 30;
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
  font-size: 1.4rem;
  line-height: 1;
  filter: drop-shadow(0 1px 1px rgba(0,0,0,0.2));
}

.token-svg {
  width: 22px;
  height: 22px;
  filter: drop-shadow(0 1px 2px rgba(0,0,0,0.3));
}

.token-pedestal {
  position: absolute;
  bottom: 2px;
  width: 22px;
  height: 8px;
  background: var(--token-color);
  border-radius: 50%;
  border: 1.5px solid rgba(255,255,255,0.4);
  box-shadow: 
    inset 0 -2px 4px rgba(0,0,0,0.3),
    inset 0 2px 4px rgba(255,255,255,0.4);
}

.token-shadow {
  position: absolute;
  bottom: -1px;
  width: 24px;
  height: 6px;
  background: rgba(0,0,0,0.5);
  border-radius: 50%;
  filter: blur(2px);
  z-index: 1;
}
</style>
