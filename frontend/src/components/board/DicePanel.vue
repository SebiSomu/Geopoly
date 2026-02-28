<script setup lang="ts">
import { computed } from 'vue'
import GameDice from './GameDice.vue'
import GameToken from './GameToken.vue'

interface Player {
  name: string
  character: 'seal' | 'capybara' | 'cat' | 'dog'
}

interface DuelData {
  challengerIdx: number
  targetIdx: number
  challengerDie1: number | null
  challengerDie2: number | null
  targetDie1: number | null
  targetDie2: number | null
}

interface Props {
  // Common Props
  isRolling?: boolean
  isMoving?: boolean
  forcedDealActive?: boolean
  username?: string
  players: Player[]
  
  // Normal Mode Props
  diceValue1?: number
  diceValue2?: number
  isMyTurn?: boolean
  currentPlayerName?: string
  
  // Duel Mode Props
  isDuel?: boolean
  duelData?: DuelData | null
  
  // Reroll Dice Props
  isRerollDice?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  diceValue1: 1,
  diceValue2: 3,
  isRolling: false,
  isMoving: false,
  forcedDealActive: false,
  isMyTurn: false,
  isDuel: false,
  duelData: null,
  isRerollDice: false,
  players: () => [],
  username: ''
})

const emit = defineEmits<{
  (e: 'roll'): void
  (e: 'rollDuel'): void
  (e: 'finish-duel'): void
}>()

// Duel Helpers
const challenger = computed(() => props.players && props.duelData ? props.players[props.duelData.challengerIdx] : null)
const target = computed(() => props.players && props.duelData ? props.players[props.duelData.targetIdx] : null)

const nextRollerIdx = computed(() => {
  if (!props.duelData) return -1
  if (props.duelData.challengerDie1 === null) return props.duelData.challengerIdx
  if (props.duelData.targetDie1 === null) return props.duelData.targetIdx
  return -1
})

const isMyTurnToRollDuel = computed(() => {
  const roller = props.players && nextRollerIdx.value !== -1 ? props.players[nextRollerIdx.value] : null
  return roller && roller.name === props.username
})

const nextRollerName = computed(() => {
  if (props.isDuel) {
    const roller = props.players && nextRollerIdx.value !== -1 ? props.players[nextRollerIdx.value] : null
    return roller ? roller.name : 'Nobody'
  }
  return props.currentPlayerName
})

const isWinner = (idx: number) => {
  if (props.duelData && props.duelData.challengerDie1 !== null && props.duelData.targetDie1 !== null) {
      const cSum = (props.duelData.challengerDie1 || 0) + (props.duelData.challengerDie2 || 0)
      const tSum = (props.duelData.targetDie1 || 0) + (props.duelData.targetDie2 || 0)
      if (idx === props.duelData.challengerIdx) return cSum > tSum
      if (idx === props.duelData.targetIdx) return tSum > cSum
  }
  return false
}

const isTie = computed(() => {
    if (!props.duelData || props.duelData.challengerDie1 === null || props.duelData.targetDie1 === null) return false
    const cSum = (props.duelData.challengerDie1 || 0) + (props.duelData.challengerDie2 || 0)
    const tSum = (props.duelData.targetDie1 || 0) + (props.duelData.targetDie2 || 0)
    return cSum === tSum
})

// Unified Dice Values
const d1 = computed(() => {
  if (props.isDuel && props.duelData) {
    // If target has rolled, show their dice. If only challenger rolled, show theirs.
    if (props.duelData.targetDie1 !== null) return props.duelData.targetDie1;
    return props.duelData.challengerDie1 || 1;
  }
  return props.diceValue1;
})
const d2 = computed(() => {
  if (props.isDuel && props.duelData) {
    if (props.duelData.targetDie2 !== null) return props.duelData.targetDie2;
    return props.duelData.challengerDie2 || 1;
  }
  return props.diceValue2;
})

const handleRollClick = () => {
  if (props.isDuel) {
    emit('rollDuel')
  } else {
    emit('roll')
  }
}

const handleFinishDuel = () => {
  emit('finish-duel')
}

const isButtonDisabled = computed(() => {
  if (props.isDuel) {
    return !isMyTurnToRollDuel.value || props.isRolling
  }
  return !props.isMyTurn || props.isRolling || props.isMoving || props.forcedDealActive
})

const buttonText = computed(() => {
  if (props.isRerollDice) return 'ROLL REROLL DICE'
  return props.isDuel ? 'ROLL DICE' : 'ROLL'
})
</script>

<template>
  <div class="dice-panel-wrapper" :class="{ 'duel-mode': isDuel, 'reroll-mode': isRerollDice }">
    <div class="dice-control-panel">
      <!-- Title for Reroll Dice -->
      <div v-if="isRerollDice" class="reroll-title-area">
        <h2 class="reroll-heading">🎲 REROLL DICE 🎲</h2>
        <div class="reroll-subtitle">Chance Card: Reroll one die!</div>
      </div>
      
      <!-- Title for Duel -->
      <div v-if="isDuel" class="duel-title-area">
        <h2 class="duel-heading">⚔️ DICE DUEL ⚔️</h2>
        <div v-if="isTie" class="tie-badge">TIE!</div>
      </div>

      <!-- Duel Participants Info -->
      <div v-if="isDuel" class="duel-participants">
        <div class="duelist-mini" :class="{ 'active': nextRollerIdx === duelData?.challengerIdx, 'winner': isWinner(duelData?.challengerIdx ?? -1) }">
          <GameToken :type="challenger?.character || 'cat'" class="mini-token" />
          <span class="mini-name">{{ challenger?.name }}</span>
        </div>
        <div class="vs-divider">VS</div>
        <div class="duelist-mini" :class="{ 'active': nextRollerIdx === duelData?.targetIdx, 'winner': isWinner(duelData?.targetIdx ?? -1) }">
          <GameToken :type="target?.character || 'cat'" class="mini-token" />
          <span class="mini-name">{{ target?.name }}</span>
        </div>
      </div>

      <!-- Reroll Dice (Single Die) -->
      <GameDice
        v-if="isRerollDice"
        :value1="d1"
        :value2="0"
        :isRolling="isRolling"
        :forcedDeal="false"
      />
      
      <!-- Normal or Duel Dice (Two Dice) -->
      <GameDice 
        v-else
        :value1="d1" 
        :value2="d2" 
        :isRolling="isRolling"
        :forcedDeal="!isDuel && diceValue1 === 1"
      />
      
      <button 
        v-if="nextRollerIdx !== -1 || !isDuel"
        class="roll-button" 
        :class="{ 'duel-btn': isDuel }"
        @click="handleRollClick"
        :disabled="isButtonDisabled"
      >
        <span class="roll-icon">🎲</span>
        <span class="roll-text">{{ buttonText }}</span>
      </button>

      <!-- Finish Duel Button -->
      <button 
        v-if="isDuel && nextRollerIdx === -1"
        class="roll-button finish-btn" 
        @click="handleFinishDuel"
      >
        <span class="roll-icon">🏁</span>
        <span class="roll-text">FINISH DUEL</span>
      </button>

      <!-- Status Indicator -->
      <div class="turn-indicator">
        <template v-if="isRerollDice">
          <span v-if="isMyTurn">Reroll your dice!</span>
          <span v-else>{{ currentPlayerName }} is rerolling...</span>
        </template>
        <template v-else-if="isDuel">
          <span v-if="nextRollerIdx !== -1">
            {{ isMyTurnToRollDuel ? 'Your' : nextRollerName + "'s" }} Turn
          </span>
          <span v-else class="duel-finished">Duel Complete!</span>
        </template>
        <template v-else>
          {{ currentPlayerName }}'s Turn <span v-if="isMyTurn">(You)</span>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dice-panel-wrapper {
  display: flex;
  justify-content: center;
  align-items: center;
  width: 100%;
}

.dice-control-panel {
  background: rgba(15, 23, 42, 0.9);
  backdrop-filter: blur(16px);
  padding: 14px;
  border-radius: 20px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  box-shadow: 
    0 15px 40px rgba(0, 0, 0, 0.5),
    inset 0 0 15px rgba(255, 255, 255, 0.05);
  animation: panelPop 0.5s cubic-bezier(0.175, 0.885, 0.32, 1.275) forwards;
  min-width: 160px;
}

@keyframes panelPop {
  from { opacity: 0; transform: scale(0.9) translateY(20px); }
  to { opacity: 1; transform: scale(1) translateY(0); }
}

/* Duel specific panel styles */
.duel-mode .dice-control-panel {
  border-color: rgba(245, 194, 231, 0.3);
  box-shadow: 
    0 30px 70px rgba(0, 0, 0, 0.6),
    0 0 30px rgba(245, 194, 231, 0.1);
}

/* Reroll specific panel styles */
.reroll-mode .dice-control-panel {
  border-color: rgba(166, 227, 161, 0.3);
  box-shadow: 
    0 30px 70px rgba(0, 0, 0, 0.6),
    0 0 30px rgba(166, 227, 161, 0.1);
}

.reroll-title-area {
  margin-bottom: -8px;
  text-align: center;
}

.reroll-heading {
  font-size: 1.1rem;
  font-weight: 900;
  color: #a6e3a1;
  letter-spacing: 4px;
  margin: 0;
  text-shadow: 0 0 20px rgba(166, 227, 161, 0.4);
}

.reroll-subtitle {
  color: #94a3b8;
  font-size: 0.8rem;
  font-weight: 600;
  margin-top: 4px;
}

.duel-title-area {
  margin-bottom: -8px;
  text-align: center;
}

.duel-heading {
  font-size: 1.1rem;
  font-weight: 900;
  color: #f5c2e7;
  letter-spacing: 4px;
  margin: 0;
  text-shadow: 0 0 20px rgba(245, 194, 231, 0.4);
}

.tie-badge {
  background: #fab387;
  color: #11111b;
  padding: 4px 12px;
  border-radius: 6px;
  font-size: 0.8rem;
  font-weight: 800;
  display: inline-block;
  margin-top: 8px;
}

.duel-participants {
  display: flex;
  align-items: center;
  gap: 12px;
  background: rgba(255, 255, 255, 0.04);
  padding: 6px 14px;
  border-radius: 12px;
}

.duelist-mini {
  display: flex;
  align-items: center;
  gap: 8px;
  opacity: 0.6;
  transition: all 0.3s ease;
}

.duelist-mini.active {
  opacity: 1;
  transform: scale(1.1);
}

.duelist-mini.winner {
  color: #a6e3a1;
}

.mini-token {
  width: 24px;
  height: 24px;
}

.mini-name {
  font-size: 0.9rem;
  font-weight: 700;
}

.vs-divider {
  font-weight: 900;
  font-style: italic;
  color: rgba(255, 255, 255, 0.2);
}

.roll-button {
  background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
  color: white;
  border: none;
  padding: 8px 24px;
  font-size: 1rem;
  font-weight: 800;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275);
  display: flex;
  align-items: center;
  gap: 8px;
  text-transform: uppercase;
  letter-spacing: 2px;
  box-shadow: 0 10px 20px rgba(37, 99, 235, 0.3);
}

.roll-button.duel-btn {
  background: linear-gradient(135deg, #f5c2e7 0%, #eba0ac 100%);
  color: #11111b;
  box-shadow: 0 12px 24px rgba(245, 194, 231, 0.3);
}

.roll-button.finish-btn {
  background: linear-gradient(135deg, #a6e3a1 0%, #94e2d5 100%);
  color: #11111b;
  box-shadow: 0 12px 24px rgba(166, 227, 161, 0.3);
}

.roll-button:hover:not(:disabled) {
  transform: translateY(-4px) scale(1.03);
  filter: brightness(1.1);
}

.roll-button:active:not(:disabled) {
  transform: translateY(0) scale(0.96);
}

.roll-button:disabled {
  background: #334155;
  color: #475569;
  cursor: not-allowed;
  opacity: 0.6;
  box-shadow: none;
  transform: none;
}

.turn-indicator {
  font-size: 0.8rem;
  font-weight: 600;
  color: #94a3b8;
  letter-spacing: 0.5px;
}

.turn-indicator span {
  color: #3b82f6;
}

.duel-mode .turn-indicator span {
  color: #f5c2e7;
}

.duel-finished {
  color: #a6e3a1 !important;
  font-weight: 800;
}

@media (max-width: 600px) {
  .dice-control-panel {
    padding: 20px;
    gap: 16px;
    min-width: 280px;
  }
  .roll-button {
    padding: 12px 36px;
    font-size: 1.2rem;
  }
}
</style>
