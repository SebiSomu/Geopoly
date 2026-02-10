<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  players: Array<{ name: string; character: string }>;
  duelData: { challengerIdx: number; targetIdx: number; challengerRoll: number | null; targetRoll: number | null };
  username: string;
}>()

const emit = defineEmits<{
  (e: 'roll'): void
}>()

const challenger = computed(() => props.players[props.duelData.challengerIdx]);
const target = computed(() => props.players[props.duelData.targetIdx]);

const nextRollerIdx = computed(() => {
  if (props.duelData.challengerRoll === null) return props.duelData.challengerIdx;
  if (props.duelData.targetRoll === null) return props.duelData.targetIdx;
  return -1;
});

const isMyTurnToRoll = computed(() => {
  const roller = props.players[nextRollerIdx.value];
  return roller && roller.name === props.username;
});

const nextRollerName = computed(() => {
  const roller = props.players[nextRollerIdx.value];
  return roller ? roller.name : 'Nobody';
});

const isWinner = (idx: number) => {
  if (props.duelData.challengerRoll !== null && props.duelData.targetRoll !== null) {
      if (idx === props.duelData.challengerIdx) return props.duelData.challengerRoll > props.duelData.targetRoll;
      if (idx === props.duelData.targetIdx) return props.duelData.targetRoll > props.duelData.challengerRoll;
  }
  return false;
};

const isTie = computed(() => {
    return props.duelData.challengerRoll !== null && 
           props.duelData.targetRoll !== null && 
           props.duelData.challengerRoll === props.duelData.targetRoll;
});
</script>

<template>
  <div class="modal-overlay">
    <div class="modal-content duel-modal">
      <div class="duel-header">
         <div class="duel-title">DICE DUEL</div>
         <div v-if="isTie" class="tie-badge">TIE! ROLL AGAIN</div>
      </div>

      <div class="arena">
        <!-- Challenger -->
        <div v-if="challenger" class="duelist" :class="{ 'active': nextRollerIdx === duelData.challengerIdx, 'winner': isWinner(duelData.challengerIdx) }">
          <div class="player-label">CHALLENGER</div>
          <div class="avatar-box">
             <div class="avatar-fallback">{{ challenger.name.charAt(0) }}</div>
          </div>
          <div class="player-name">{{ challenger.name }}</div>
          <div class="roll-display" :class="{ 'rolled': duelData.challengerRoll !== null }">
            {{ duelData.challengerRoll !== null ? duelData.challengerRoll : '?' }}
          </div>
        </div>

        <div class="vs-container">
          <div class="vs-text">VS</div>
        </div>

        <!-- Target -->
        <div v-if="target" class="duelist" :class="{ 'active': nextRollerIdx === duelData.targetIdx, 'winner': isWinner(duelData.targetIdx) }">
          <div class="player-label">OPPONENT</div>
          <div class="avatar-box">
             <div class="avatar-fallback">{{ target.name.charAt(0) }}</div>
          </div>
          <div class="player-name">{{ target.name }}</div>
          <div class="roll-display" :class="{ 'rolled': duelData.targetRoll !== null }">
            {{ duelData.targetRoll !== null ? duelData.targetRoll : '?' }}
          </div>
        </div>
      </div>

      <div class="duel-footer">
        <button 
          v-if="isMyTurnToRoll" 
          @click="emit('roll')" 
          class="roll-btn"
        >
          ROLL DIE
        </button>
        <div v-else-if="nextRollerIdx !== -1" class="waiting-box">
          <div class="spinner"></div>
          <span>Waiting for {{ nextRollerName }} to roll...</span>
        </div>
        <div v-else class="result-box">
            Duel Complete!
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
  background: rgba(0, 0, 0, 0.9);
  backdrop-filter: blur(10px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1100;
}

.duel-modal {
  background: #11111b;
  border: 4px solid #313244;
  border-radius: 32px;
  padding: 60px;
  width: 90%;
  max-width: 800px;
  text-align: center;
  box-shadow: 0 0 100px rgba(0, 0, 0, 0.8), 0 0 20px rgba(245, 194, 231, 0.1);
}

.duel-title {
  font-size: 4rem;
  font-weight: 900;
  color: #f5c2e7;
  letter-spacing: 10px;
  margin-bottom: 40px;
  text-shadow: 0 0 30px rgba(245, 194, 231, 0.4);
}

.tie-badge {
    background: #fab387;
    color: #11111b;
    padding: 8px 16px;
    border-radius: 8px;
    font-weight: bold;
    display: inline-block;
    margin-bottom: 20px;
}

.arena {
  display: flex;
  align-items: center;
  justify-content: space-around;
  margin-bottom: 60px;
}

.duelist {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
  padding: 30px;
  border-radius: 24px;
  background: rgba(255, 255, 255, 0.02);
  transition: all 0.3s ease;
  width: 220px;
}

.duelist.active {
  background: rgba(245, 194, 231, 0.05);
  box-shadow: 0 0 30px rgba(245, 194, 231, 0.15);
  transform: translateY(-10px);
}

.duelist.winner {
    background: rgba(166, 227, 161, 0.1);
    box-shadow: 0 0 30px rgba(166, 227, 161, 0.2);
    border: 2px solid #a6e3a1;
}

.player-label {
  font-size: 0.8rem;
  font-weight: 800;
  color: #9399b2;
  letter-spacing: 2px;
}

.avatar-box {
  width: 100px;
  height: 100px;
  background: #313244;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 4px solid rgba(255, 255, 255, 0.1);
}

.avatar-fallback {
  font-size: 3rem;
  font-weight: bold;
  color: #cdd6f4;
}

.player-name {
  font-size: 1.8rem;
  font-weight: 700;
  color: #f5f5f7;
}

.roll-display {
  font-size: 5rem;
  font-weight: 900;
  color: #9399b2;
  width: 120px;
  height: 120px;
  background: #181825;
  border-radius: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 3px dashed rgba(255, 255, 255, 0.1);
}

.roll-display.rolled {
  color: #f5c2e7;
  border-style: solid;
  border-color: rgba(245, 194, 231, 0.3);
  animation: rollIn 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
}

.vs-container {
  display: flex;
  align-items: center;
  justify-content: center;
}

.vs-text {
  font-size: 3rem;
  font-weight: 900;
  color: #585b70;
  font-style: italic;
}

.duel-footer {
  min-height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.roll-btn {
  background: #f5c2e7;
  color: #11111b;
  border: none;
  padding: 20px 60px;
  font-size: 1.8rem;
  font-weight: 800;
  border-radius: 16px;
  cursor: pointer;
  transition: all 0.2s;
  box-shadow: 0 10px 20px rgba(245, 194, 231, 0.2);
}

.roll-btn:hover {
  transform: translateY(-4px) scale(1.05);
  box-shadow: 0 15px 30px rgba(245, 194, 231, 0.3);
}

.waiting-box {
  display: flex;
  align-items: center;
  gap: 16px;
  color: #9399b2;
  font-size: 1.2rem;
}

.spinner {
  width: 24px;
  height: 24px;
  border: 3px solid rgba(147, 153, 178, 0.2);
  border-top-color: #f5c2e7;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

@keyframes rollIn {
  from { transform: scale(0.5) rotate(-45deg); opacity: 0; }
  to { transform: scale(1) rotate(0); opacity: 1; }
}
</style>
