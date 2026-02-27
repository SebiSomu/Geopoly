<script setup lang="ts">
const props = defineProps<{
  stamps: Array<{
    name: string;
    color: string;
    diameter: number;
    destination_id?: number | null;
    size: number;
    price: number;
  }>;
  title?: string;
  subtitle?: string;
}>()

const emit = defineEmits<{
  (e: 'select', name: string): void;
}>()

// Color map matching Stamp.vue COLORS
const COLOR_MAP: Record<string, string> = {
  grey:      '#bdbdbd',
  brown:     '#8B4513',
  lightblue: '#79F7EF',
  pink:      '#D61A8B',
  orange:    '#F0760C',
  red:       '#CC0000',
  yellow:    '#FBFF00',
  green:     '#04910D',
  darkblue:  '#0D47A1',
  blue:      '#0D47A1',
}

const stampColor = (color: string) => COLOR_MAP[color.toLowerCase()] ?? '#bdbdbd'
</script>

<template>
  <div class="modal-overlay">
    <div class="modal-content">
      <div class="modal-header">
        <div class="header-icon">💰</div>
        <h2>{{ title || 'Stamp Amnesty' }}</h2>
        <div class="subtitle">{{ subtitle || 'Choose a stamp to sell for 150% its value' }}</div>
      </div>
      
      <div class="stamps-list">
        <button 
          v-for="stamp in stamps" 
          :key="stamp.name"
          class="stamp-row"
          @click="emit('select', stamp.name)"
        >
          <!-- Colored circle preview -->
          <div class="stamp-circle" :style="{ background: stampColor(stamp.color) }">
            <svg viewBox="0 0 100 100" style="width:100%;height:100%">
              <circle cx="50" cy="50" r="48" fill="none" stroke="rgba(0,0,0,0.25)" stroke-width="2"/>
              <text
                v-if="stamp.destination_id"
                x="50" y="60"
                font-size="30" font-weight="900"
                text-anchor="middle"
                fill="rgba(0,0,0,0.65)"
                font-family="Arial Black, sans-serif"
              >{{ stamp.destination_id }}</text>
              <text v-else x="50" y="63" font-size="40" text-anchor="middle" fill="rgba(0,0,0,0.6)">★</text>
            </svg>
          </div>

          <div class="stamp-info">
            <div class="sname">{{ stamp.name }}</div>
            <div class="sprice">Sell for M{{ Math.floor(stamp.price * 1.5) }}</div>
          </div>

          <div class="sell-tag">SELL</div>
        </button>

        <div v-if="stamps.length === 0" class="no-stamps">
          No stamps in passport.
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.9);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 3000;
}

.modal-content {
  background: #0d1b31;
  border: 2px solid #fbbf24;
  border-radius: 24px;
  padding: 28px 24px 24px;
  width: 92%;
  max-width: 400px;
  box-shadow: 0 0 40px rgba(251,191,36,0.15);
  display: flex;
  flex-direction: column;
  gap: 0;
}

.modal-header {
  text-align: center;
  margin-bottom: 20px;
}

.header-icon { font-size: 2.2rem; margin-bottom: 6px; }

h2 {
  font-family: 'Oswald', sans-serif;
  color: #fbbf24;
  font-size: 1.8rem;
  margin: 0 0 6px;
  text-transform: uppercase;
  letter-spacing: 2px;
}

.subtitle {
  color: rgba(255,255,255,0.6);
  font-size: 0.9rem;
}

.stamps-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  max-height: 380px;
  overflow-y: auto;
  padding-right: 4px;
}

.stamps-list::-webkit-scrollbar { width: 5px; }
.stamps-list::-webkit-scrollbar-thumb {
  background: rgba(251,191,36,0.3);
  border-radius: 3px;
}

.stamp-row {
  display: flex;
  align-items: center;
  gap: 14px;
  background: rgba(30,41,59,0.6);
  border: 1px solid rgba(251,191,36,0.12);
  border-radius: 14px;
  padding: 10px 14px;
  cursor: pointer;
  transition: background 0.15s, border-color 0.15s, transform 0.12s;
  width: 100%;
  text-align: left;
}

.stamp-row:hover {
  background: rgba(251,191,36,0.1);
  border-color: #fbbf24;
  transform: translateX(3px);
}

.stamp-circle {
  width: 44px;
  height: 44px;
  border-radius: 50%;
  flex-shrink: 0;
  border: 1px solid rgba(0,0,0,0.2);
  box-shadow: 0 2px 6px rgba(0,0,0,0.3);
}

.stamp-info { flex: 1; overflow: hidden; }

.sname {
  font-family: 'Oswald', sans-serif;
  font-size: 1.05rem;
  font-weight: 600;
  color: #fff;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.sprice {
  font-size: 0.72rem;
  color: #fbbf24;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.4px;
  margin-top: 2px;
}

.sell-tag {
  font-family: 'Oswald', sans-serif;
  font-size: 0.8rem;
  font-weight: 700;
  color: #fbbf24;
  background: rgba(251,191,36,0.1);
  border: 1px solid rgba(251,191,36,0.25);
  border-radius: 8px;
  padding: 5px 12px;
  letter-spacing: 1px;
  flex-shrink: 0;
  transition: background 0.15s, color 0.15s;
}

.stamp-row:hover .sell-tag {
  background: #fbbf24;
  color: #0c1828;
}

.no-stamps {
  text-align: center;
  padding: 30px;
  color: rgba(255,255,255,0.45);
  font-style: italic;
}
</style>
