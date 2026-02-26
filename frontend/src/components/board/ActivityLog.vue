<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  entries: Array<{ playerIdx: number | null, message: string }>
  players: Array<{ name: string, character: string }>
}>()

const CHARACTER_EMOJIS: Record<string, string> = {
  cat: '🐱',
  seal: '🦭',
  capybara: '🦫',
  dog: '🐶',
}

const COLOR_MAP: Record<string, string> = {
  Brown: '#8b4513',
  LightBlue: '#add8e6',
  Pink: '#ffc0cb',
  Orange: '#ffa500',
  Yellow: '#ffff00',
  Red: '#ff0000',
  Green: '#008000',
  DarkBlue: '#00008b'
};

function getPlayerColor(idx: number): string {
  const colors = ['#60a5fa', '#f472b6', '#34d399', '#fbbf24']
  return colors[idx % colors.length] ?? '#94a3b8'
}

const displayEntries = computed(() => {
  return props.entries.map(entry => {
    let playerName = ''
    let playerEmoji = ''
    let playerCharacter: string | null = null
    let playerColor = '#94a3b8'
    
    if (entry.playerIdx !== null && entry.playerIdx !== undefined) {
      const p = props.players[entry.playerIdx]
      if (p) {
        playerName = p.name
        playerCharacter = p.character
        playerEmoji = CHARACTER_EMOJIS[p.character ?? ''] || '🎮'
        playerColor = getPlayerColor(entry.playerIdx)
      }
    }

    // Parse message for dots: [DOT:ColorName]
    const contentMessage = playerName ? entry.message.replace(playerName, '').trim() : entry.message;
    const parts = contentMessage.split(/(\[DOT:\w+\])/g);
    const parsedMessage = parts.map(part => {
      const match = part.match(/\[DOT:(\w+)\]/);
      if (match && match[1]) {
        const colorName = match[1];
        return { type: 'dot', color: COLOR_MAP[colorName] || '#fff' };
      }
      return { type: 'text', content: part };
    });
    
    return {
      ...entry,
      playerName,
      playerEmoji,
      playerCharacter,
      playerColor,
      parsedMessage
    }
  })
})
</script>

<template>
  <div class="activity-log" v-if="entries.length > 0">
    <div class="log-header">
      <span class="log-icon">📋</span>
      <span>Activity Log</span>
    </div>
    <TransitionGroup name="log-item" tag="div" class="log-entries">
      <div
        v-for="(entry, index) in displayEntries"
        :key="entry.message + index"
        class="log-entry"
      >
        <div class="entry-icon">
          <template v-if="entry.playerCharacter === 'seal'">
            <svg viewBox="0 0 24 24" class="token-svg">
              <circle cx="12" cy="13" r="8" fill="#E0E0E0" />
              <circle cx="9" cy="12" r="1.2" fill="#333" />
              <circle cx="15" cy="12" r="1.2" fill="#333" />
              <ellipse cx="12" cy="16" rx="3.5" ry="2.5" fill="#F5F5F5" />
              <circle cx="12" cy="15.5" r="1" fill="#333" />
              <path d="M8 16L5 15.5M8 16.5L5 17.5" stroke="#999" stroke-width="0.5" />
              <path d="M16 16L19 15.5M16 16.5L19 17.5" stroke="#999" stroke-width="0.5" />
            </svg>
          </template>
          <template v-else-if="entry.playerCharacter === 'capybara'">
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
          <span v-else class="entry-emoji">{{ entry.playerEmoji || '🏦' }}</span>
        </div>
        <span class="entry-text">
          <span class="entry-player" :style="{ color: entry.playerColor }">{{ entry.playerName }}</span>
          <template v-for="(part, pIdx) in entry.parsedMessage" :key="pIdx">
            <span v-if="part.type === 'dot'" class="log-dot" :style="{ backgroundColor: part.color }"></span>
            <span v-else>{{ part.content }}</span>
          </template>
        </span>
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.activity-log {
  margin-top: 12px;
  background: #0d1628; /* Solid background for performance */
  border: 1px solid rgba(148, 163, 184, 0.15);
  border-radius: 12px;
  padding: 10px 12px;
  max-width: 300px; /* Wider */
  width: 100%;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
}

.log-header {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 10px;
  font-weight: 600;
  color: rgba(148, 163, 184, 0.8);
  text-transform: uppercase;
  letter-spacing: 0.8px;
  padding-bottom: 6px;
  border-bottom: 1px solid rgba(148, 163, 184, 0.1);
  margin-bottom: 4px;
}

.log-icon {
  font-size: 11px;
}

.log-entries {
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-height: 190px; /* Taller to show more entries */
  overflow-y: auto;
  scrollbar-width: thin;
  scrollbar-color: rgba(148, 163, 184, 0.2) transparent;
}

.log-entries::-webkit-scrollbar {
  width: 4px;
}

.log-entries::-webkit-scrollbar-thumb {
  background: rgba(148, 163, 184, 0.2);
  border-radius: 2px;
}

.log-entry {
  display: flex;
  align-items: flex-start;
  gap: 6px;
  padding: 3px 4px;
  border-radius: 4px;
  transition: background 0.2s ease;
  font-size: 10.5px;
  line-height: 1.4;
}

.log-entry:hover {
  background: rgba(148, 163, 184, 0.08);
}

.entry-icon {
  flex-shrink: 0;
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-top: 1px;
}

.token-svg {
  width: 16px;
  height: 16px;
}

.entry-emoji {
  font-size: 12px;
}

.entry-text {
  color: rgba(226, 232, 240, 0.85);
  word-break: break-word;
}

.entry-player {
  font-weight: 600;
  margin-right: 4px;
}

.log-dot {
  display: inline-block;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  margin: 0 4px;
  vertical-align: middle;
  border: 1px solid rgba(255, 255, 255, 0.2);
  box-shadow: 0 0 4px rgba(0, 0, 0, 0.5);
}

/* TransitionGroup animations */
.log-item-enter-active {
  transition: all 0.3s ease-out;
}

.log-item-leave-active {
  transition: all 0.2s ease-in;
}

.log-item-enter-from {
  opacity: 0;
  transform: translateY(-8px);
}

.log-item-leave-to {
  opacity: 0;
  transform: translateX(-10px);
}

.log-item-move {
  transition: transform 0.3s ease;
}
</style>
