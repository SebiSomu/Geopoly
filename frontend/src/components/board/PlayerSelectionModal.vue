<script setup lang="ts">
import { computed } from 'vue'
import GameToken from './GameToken.vue'

const props = defineProps<{
  players: Array<{ 
    name: string; 
    money: number; 
    properties: any[]; 
    character: 'seal' | 'capybara' | 'cat' | 'dog' 
  }>;
  selectorIdx: number;
  action: string;
  username: string;
}>()

const emit = defineEmits<{
  (e: 'select', name: string): void
}>()

const targetablePlayers = computed(() => {
  return props.players.filter((p, idx) => {
    if (idx === props.selectorIdx) return false;
    
    // Pentru acțiuni ce implică ștampile, ținta trebuie să aibă cel puțin una
    if (['SwapStamps', 'swap_stamps', 'StealStampAndPay', 'SneakySwap', 'StealFirstClass'].includes(props.action)) {
      return p.properties && p.properties.length > 0;
    }
    
    return true;
  });
});

const title = computed(() => {
  switch (props.action) {
    case 'swap_stamps':
    case 'SwapStamps': return 'Swap Stamps';
    case 'dice_challenge':
    case 'DiceDuel': return 'Dice Duel Challenge';
    case 'StealFirstClass': return 'Steal First Class';
    case 'StealStampAndPay': return 'Steal Stamp & Pay';
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
    case 'StealStampAndPay': return 'steal a stamp from';
    default: return 'target';
  }
});
</script>

<template>
  <div class="modal-overlay">
    <div class="modal-content selection-modal">
      <div class="modal-header">
        <div class="header-icon">🎯</div>
        <h2>{{ title }}</h2>
        <div class="subtitle">Choose a player to <span class="action-highlight">{{ actionText }}</span></div>
      </div>
      
      <div class="players-grid">
        <div 
          v-for="player in targetablePlayers" 
          :key="player.name"
          class="player-card"
          @click="emit('select', player.name)"
        >
          <div class="player-avatar">
             <div class="token-container">
                <GameToken :type="player.character" />
             </div>
          </div>
          <div class="player-info">
            <div class="name">{{ player.name }}</div>
            <div class="stats">
              <span class="money">G{{ player.money }}</span>
              <span class="separator">•</span>
              <span class="stamps">{{ player.properties.length }} stamps</span>
            </div>
          </div>
          <div class="select-btn">SELECT</div>
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
  background: rgba(0, 0, 0, 0.9); /* Solid opaque black for zero render cost */
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
}

.modal-content {
  background: #0d1b31; /* Solid background is faster than gradients + filters */
  border: 1px solid rgba(255, 215, 0, 0.3);
  border-radius: 28px;
  padding: 35px;
  width: 90%;
  max-width: 480px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.4);
}

.modal-header {
  text-align: center;
  margin-bottom: 30px;
}

.header-icon {
  font-size: 2.5rem;
  margin-bottom: 10px;
}

h2 {
  font-family: 'Oswald', sans-serif;
  color: #FFD700;
  font-size: 2.2rem;
  margin: 0 0 10px 0;
  text-transform: uppercase;
  letter-spacing: 3px;
  text-shadow: 0 2px 10px rgba(255, 215, 0, 0.2);
}

.subtitle {
  color: rgba(255, 255, 255, 0.7);
  font-size: 1.05rem;
}

.action-highlight {
  color: #fff;
  font-weight: 600;
}

.players-grid {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.player-card {
  display: flex;
  align-items: center;
  padding: 16px 20px;
  background: #13243d;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 18px;
  cursor: pointer;
  transition: transform 0.1s ease;
  position: relative;
}

.player-card:hover {
  background: rgba(255, 215, 0, 0.1);
  transform: translateY(-2px);
}

.player-avatar {
  margin-right: 18px;
  flex-shrink: 0;
}

.token-container {
  width: 54px;
  height: 54px;
  background: #1a2a44;
  border-radius: 50%;
  border: 1px solid rgba(255, 255, 255, 0.1);
  display: flex;
  align-items: center;
  justify-content: center;
  --token-size: 44px;
}
/* Removed :deep selectors that can be slow */

.player-info {
  flex: 1;
}

.name {
  font-family: 'Oswald', sans-serif;
  font-size: 1.3rem;
  font-weight: 600;
  color: #fff;
  margin-bottom: 2px;
  letter-spacing: 0.5px;
}

.stats {
  font-size: 0.95rem;
  color: rgba(255, 255, 255, 0.5);
}

.separator {
  margin: 0 8px;
  opacity: 0.3;
}

.money {
  color: #FFD700;
  font-weight: 700;
}

.select-btn {
  font-family: 'Oswald', sans-serif;
  font-size: 0.85rem;
  font-weight: 700;
  color: #FFD700;
  background: rgba(255, 215, 0, 0.1);
  padding: 6px 14px;
  border-radius: 10px;
  border: 1px solid rgba(255, 215, 0, 0.2);
  letter-spacing: 1px;
  opacity: 0.7;
  transition: all 0.2s ease;
}

.player-card:hover .select-btn {
  opacity: 1;
  background: #FFD700;
  color: #0c1828;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes modalPop {
  0% { transform: scale(0.9) translateY(20px); opacity: 0; }
  100% { transform: scale(1) translateY(0); opacity: 1; }
}
</style>
