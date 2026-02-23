<script setup lang="ts">
import { computed } from 'vue';
import { useMutation } from '@vue/apollo-composable';
import { USE_CARD_MUTATION } from '../graphql/operations';

const props = defineProps<{
  code: string;
  username: string;
  hereAndNowCards: any[];
  chanceCards: any[];
  isMyTurn: boolean;
  inJail: boolean;
  propertyCount: number;
  players: any[];
}>();

const emit = defineEmits(['refresh']);

const { mutate: useCardMutate } = useMutation(USE_CARD_MUTATION);

const hasCards = computed(() => props.hereAndNowCards.length > 0 || props.chanceCards.length > 0);

const useCard = async (cardId: string) => {
  try {
    await useCardMutate({
      code: props.code,
      username: props.username,
      cardId
    });
    emit('refresh');
  } catch (e) {
    console.error("Failed to use card:", e);
  }
};

const getCardStatus = (card: any) => {
  const desc = card.description.toLowerCase();
  
  // Movement cards
  if (desc.includes('advance 5 spaces') || desc.includes('go to any space')) {
    if (!props.isMyTurn) return { playable: false, hint: 'Not your turn' };
  }

  // Jail cards
  if (desc.includes('out of jail')) {
    if (!props.inJail) return { playable: false, hint: 'Not in jail' };
  }

  // Swap cards / Take all stamps
  if (desc.includes('swap your last stamp') || desc.includes('swap their last stamps') || desc.includes('take the last stamp from all')) {
    const hasOthersWithStamps = props.players.some(p => p.username !== props.username && p.properties.length > 0);
    if (!hasOthersWithStamps) return { playable: false, hint: 'No stamps to take/swap' };
    
    // For personal swap, also need own stamps
    if (desc.includes('swap your last stamp') && props.propertyCount === 0) {
      return { playable: false, hint: 'No stamps to swap' };
    }
  }

  // Richest player
  if (desc.includes('player with the most stamps pays you')) {
    const opponentStamps = props.players
      .filter(p => p.username !== props.username)
      .map(p => p.properties.length);
    const maxOpponentStamps = opponentStamps.length > 0 ? Math.max(...opponentStamps) : 0;
    
    if (maxOpponentStamps === 0) return { playable: false, hint: 'No one has stamps' };
    if (maxOpponentStamps <= props.propertyCount) return { playable: false, hint: 'You are the richest' };
  }

  // Just Say No
  if (desc.includes('counter another player\'s action') || desc.includes('just say no')) {
    const me = props.players.find(p => p.username === props.username);
    if (me && !me.canUseSayNo) {
      return { playable: false, hint: 'Nothing to cancel' };
    }
  }

  // Intercept Purchase
  if (desc.includes('intercept the last property purchased')) {
    const me = props.players.find(p => p.username === props.username);
    if (me && !me.canUseIntercept) {
      return { playable: false, hint: 'No recent purchase by others' };
    }
  }

  // Discount Purchase (M100)
  if (desc.includes('pay only m100 for that space')) {
    const me = props.players.find(p => p.username === props.username);
    if (me && !me.canUseDiscount) {
      return { playable: false, hint: 'Not applicable now' };
    }
  }

  // Collect Tax
  if (desc.includes('collect a tourist tax instead of paying')) {
    const me = props.players.find(p => p.username === props.username);
    if (me && !me.canUseCollectTax) {
      return { playable: false, hint: 'No recent payment to player' };
    }
  }

  // Steal First Class
  if (desc.includes('steal another player\'s "first class" stamp')) {
    const me = props.players.find(p => p.username === props.username);
    if (me && !me.canUseStealFirstClass) {
      return { playable: false, hint: 'No recent First Class by others' };
    }
  }

  return { playable: true, hint: 'Tap to play' };
}

const formatHnDescription = (desc: string) => {
  const lower = desc.toLowerCase();
  if (lower.includes('advance 5 spaces') || lower.includes('go to any space on the board')) {
    // Remove trailing dot if exists
    const trimmed = desc.trim().replace(/\.$/, '');
    return `${trimmed} in this round`;
  }
  return desc;
};
</script>

<template>
  <div class="compact-hand-area" v-if="hasCards">
    <div class="portfolio-divider">
        <h3>🎴 My Cards</h3>
    </div>
    <div class="cards-list">
      <!-- Here & Now Cards -->
      <div 
        v-for="card in hereAndNowCards" 
        :key="card.id" 
        class="compact-card-row"
        :class="{ 
          'can-play': getCardStatus(card).playable,
          'disabled': !getCardStatus(card).playable 
        }" 
        @click="getCardStatus(card).playable && useCard(card.id)"
      >
        <div class="mini-card hn-mini">
            <span class="mini-card-text">HERE&NOW</span>
        </div>
        <div class="compact-card-info">
            <p class="compact-desc">{{ formatHnDescription(card.description) }}</p>
            <span class="play-hint">{{ getCardStatus(card).hint }}</span>
        </div>
      </div>

      <!-- Chance Cards -->
      <div 
        v-for="card in chanceCards" 
        :key="card.id" 
        class="compact-card-row"
        :class="{ 
          'can-play': getCardStatus(card).playable,
          'disabled': !getCardStatus(card).playable 
        }" 
        @click="getCardStatus(card).playable && useCard(card.id)"
      >
        <div class="mini-card chance-mini">
            <span class="mini-card-text">CHANCE</span>
        </div>
        <div class="compact-card-info">
            <p class="compact-desc">{{ card.description }}</p>
            <span class="play-hint">{{ getCardStatus(card).hint }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.compact-hand-area {
  margin-top: 15px;
  animation: fade-in 0.5s ease-out;
}

@keyframes fade-in {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.portfolio-divider {
  display: flex;
  align-items: center;
  margin-bottom: 10px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  padding-bottom: 5px;
}

.portfolio-divider h3 {
  color: #fbbf24;
  font-size: 0.8rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  margin: 0;
}

.cards-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.compact-card-row {
  display: flex;
  align-items: center;
  gap: 12px;
  background: rgba(30, 41, 59, 0.4);
  padding: 8px;
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.05);
  transition: all 0.2s ease;
  cursor: default;
}

.compact-card-row.can-play {
  cursor: pointer;
}

.compact-card-row.can-play:hover {
  background: rgba(30, 41, 59, 0.8);
  border-color: rgba(251, 191, 36, 0.3);
  transform: translateX(4px);
}

.compact-card-row.disabled {
  opacity: 0.5;
  filter: grayscale(0.5);
  cursor: not-allowed;
  pointer-events: none;
}

.compact-card-row.disabled .play-hint {
  color: #ef4444;
  opacity: 0.9;
}

.mini-card {
  width: 45px;
  height: 28px;
  flex-shrink: 0;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 2px 4px rgba(0,0,0,0.3);
  border: 1px solid rgba(255,255,255,0.2);
}

.hn-mini {
  background: linear-gradient(135deg, #ef4444, #b91c1c); /* Red as requested */
}

.chance-mini {
    background: linear-gradient(135deg, #f59e0b, #d97706); /* Yellow/Orange */
}

.mini-card-text {
  color: white;
  font-size: 0.45rem; /* Very small text */
  font-weight: 900;
  text-align: center;
  line-height: 1;
}

.compact-card-info {
    display: flex;
    flex-direction: column;
    flex-grow: 1;
}

.compact-desc {
  color: #e2e8f0;
  font-size: 0.7rem;
  margin: 0;
  line-height: 1.2;
}

.play-hint {
    color: #fbbf24;
    font-size: 0.55rem;
    font-weight: 700;
    text-transform: uppercase;
    margin-top: 2px;
    opacity: 0.7;
}

.compact-card-row:hover .play-hint {
    opacity: 1;
}
</style>
