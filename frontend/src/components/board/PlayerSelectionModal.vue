<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  players: Array<{ name: string; money: number; properties: any[] }>;
  selectorIdx: number;
  action: string;
  username: string;
}>()

const emit = defineEmits<{
  (e: 'select', name: string): void
}>()

const targetablePlayers = computed(() => {
  return props.players.filter((_, idx) => idx !== props.selectorIdx);
});

const title = computed(() => {
  switch (props.action) {
    case 'swap_stamps':
    case 'SwapStamps': return 'Swap Stamps';
    case 'dice_challenge':
    case 'DiceDuel': return 'Dice Duel Challenge';
    case 'StealFirstClass': return 'Steal First Class';
    default: return 'Select Player';
  }
});

const actionText = computed(() => {
  switch (props.action) {
    case 'swap_stamps':
    case 'SwapStamps': return 'swap your last stamp with';
    case 'dice_challenge':
    case 'DiceDuel': return 'challenge to a duel';
    case 'StealFirstClass': return 'steal a First Class stamp from';
    default: return 'target';
  }
});
</script>

<template>
  <div class="modal-overlay">
    <div class="modal-content selection-modal">
      <div class="modal-header">
        <h2>{{ title }}</h2>
        <div class="subtitle">Choose a player to {{ actionText }}</div>
      </div>
      
      <div class="players-grid">
        <div 
          v-for="player in targetablePlayers" 
          :key="player.name"
          class="player-card"
          @click="emit('select', player.name)"
        >
          <div class="player-avatar">
             <!-- Placeholder for character icon if needed -->
             <div class="avatar-circle">{{ player.name.charAt(0).toUpperCase() }}</div>
          </div>
          <div class="player-info">
            <div class="name">{{ player.name }}</div>
            <div class="stats">
              <span class="money">M{{ player.money }}</span>
              <span class="separator">•</span>
              <span class="stamps">{{ player.properties.length }} stamps</span>
            </div>
          </div>
          <div class="action-hint">SELECT</div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.85);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  animation: fadeIn 0.3s ease-out;
}

.modal-content {
  background: linear-gradient(135deg, #1e1e2e 0%, #11111b 100%);
  border: 2px solid rgba(255, 255, 255, 0.1);
  border-radius: 24px;
  padding: 40px;
  width: 100%;
  max-width: 500px;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
  animation: scaleIn 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.modal-header {
  text-align: center;
  margin-bottom: 32px;
}

h2 {
  color: #f5c2e7;
  font-size: 2.5rem;
  margin: 0 0 8px 0;
  text-transform: uppercase;
  letter-spacing: 2px;
  text-shadow: 0 0 20px rgba(245, 194, 231, 0.3);
}

.subtitle {
  color: #a6adc8;
  font-size: 1.1rem;
}

.players-grid {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.player-card {
  display: flex;
  align-items: center;
  padding: 20px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 16px;
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;
  overflow: hidden;
}

.player-card:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: #f5c2e7;
  transform: translateX(8px);
}

.player-avatar {
  margin-right: 20px;
}

.avatar-circle {
  width: 50px;
  height: 50px;
  background: #313244;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.5rem;
  font-weight: bold;
  color: #cdd6f4;
  border: 2px solid rgba(255, 255, 255, 0.1);
}

.player-info {
  flex: 1;
}

.name {
  font-size: 1.4rem;
  font-weight: 600;
  color: #f5f5f7;
  margin-bottom: 4px;
}

.stats {
  font-size: 1rem;
  color: #9399b2;
}

.separator {
  margin: 0 8px;
  opacity: 0.3;
}

.money {
  color: #a6e3a1;
  font-weight: 600;
}

.action-hint {
  font-size: 0.8rem;
  font-weight: 800;
  color: #f5c2e7;
  letter-spacing: 1px;
  opacity: 0;
  transition: opacity 0.2s ease;
}

.player-card:hover .action-hint {
  opacity: 1;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes scaleIn {
  from { opacity: 0; transform: scale(0.9); }
  to { opacity: 1; transform: scale(1); }
}
</style>
