<script setup lang="ts">
import GameToken from './GameToken.vue'
import Passport from './Passport.vue'
import type { PassportProperty } from './Passport.vue'

defineProps<{
  winnerName: string | null
  character: 'cat' | 'seal' | 'capybara' | 'dog'
  properties: PassportProperty[]
}>()

defineEmits<{
  (e: 'backToLobby'): void
}>()
</script>

<template>
  <div class="victory-overlay">
    <div class="modal-content victory">
      <div class="victory-header">
        <div class="trophy-icon">🏆</div>
        <h3>CONGRATULATIONS!</h3>
      </div>

      <div class="winner-main-content">
        <div class="winner-identity">
          <div class="winner-token-wrapper">
            <GameToken :type="character" class="winner-token" />
          </div>
          <h2 class="winner-name-text">{{ winnerName }}</h2>
        </div>

        <div class="winner-passport-container">
          <Passport 
            :properties="properties"
            class="compact-passport"
          />
        </div>
      </div>

      <p class="victory-subtitle">Has completed their passport and conquered the world!</p>
      
      <div class="victory-actions">
        <button class="modal-btn back-to-lobby" @click="$emit('backToLobby')">
          Return to Lobby
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.victory-overlay {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  z-index: 2000;
}

.modal-content.victory {
  background: linear-gradient(135deg, #1e1b4b 0%, #312e81 100%);
  padding: 20px;
  border-radius: 20px;
  border: 3px solid #fbbf24;
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.9);
  text-align: center;
  position: relative;
  max-width: 440px;
  width: 95%;
  margin-top: 170px; /* Increased to center perfectly on board center */
  animation: victoryPop 0.5s cubic-bezier(0.175, 0.885, 0.32, 1.275);
}

@keyframes victoryPop {
  0% { transform: scale(0.9) translateY(20px); opacity: 0; }
  100% { transform: scale(1) translateY(0); opacity: 1; }
}

.victory-header h3 {
  font-family: 'Oswald', sans-serif;
  font-size: 18px;
  color: #fbbf24;
  margin: 6px 0;
  letter-spacing: 2px;
  text-transform: uppercase;
}

.trophy-icon {
  font-size: 32px;
  margin-bottom: -5px;
}

.winner-main-content {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 20px;
  margin: 12px 0;
}

.winner-identity {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  min-width: 120px;
}

.winner-token-wrapper {
  width: 80px;
  height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 50%;
  border: 2px solid rgba(251, 191, 36, 0.3);
}

.winner-token {
  width: 65px !important;
  height: 65px !important;
}

/* Hide pedestal and shadow for "face only" look */
:deep(.winner-token .token-pedestal),
:deep(.winner-token .token-shadow) {
  display: none;
}

:deep(.winner-token .token-icon) {
  width: 100%;
  height: 100%;
}

:deep(.winner-token .emoji-face) {
  font-size: 3.2rem;
}

:deep(.winner-token .token-svg) {
  width: 100%;
  height: 100%;
}

.winner-name-text {
  font-size: 20px;
  font-weight: 800;
  color: #fff;
  margin: 0;
  max-width: 140px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.winner-passport-container {
  transform: scale(0.6);
  transform-origin: center;
  margin: -60px -50px; /* Offset the scale shrink */
}

.victory-subtitle {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.8);
  margin-bottom: 16px;
}

.modal-btn.back-to-lobby {
  background: #fbbf24;
  color: #1e1b4b;
  border: none;
  padding: 10px 24px;
  font-size: 16px;
  font-weight: 700;
  border-radius: 10px;
  cursor: pointer;
  box-shadow: 0 4px 0 #d97706;
  transition: all 0.2s ease;
}

.modal-btn.back-to-lobby:hover {
  transform: translateY(-2px);
  filter: brightness(1.1);
  box-shadow: 0 6px 0 #d97706;
}

.modal-btn.back-to-lobby:active {
  transform: translateY(2px);
  box-shadow: 0 2px 0 #d97706;
}
</style>
