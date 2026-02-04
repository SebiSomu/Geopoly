<script setup lang="ts">
import { computed } from 'vue'
import Passport from './Passport.vue'
import Stamp from './Stamp.vue'

// Exact stamp colors from Stamp.vue
const COLORS = {
  brown: '#8B4513',
  lightblue: '#79F7EF',
  pink: '#D61A8B',
  orange: '#F0760C',
  red: '#CC0000',
  yellow: '#FBFF00',
  green: '#04910D',
  darkblue: '#0D47A1',
} as const

// Board space types
interface Space {
  type: 'start' | 'destination' | 'chance' | 'airport' | 'here_and_now' | 'first_class' | 'just_visiting' | 'free_parking' | 'go_to_jail'
  name?: string
  price?: number
  tax?: number
  color?: keyof typeof COLORS
  id?: number
}

const boardSpaces: Space[] = [
  { type: 'start' },

  { type: 'destination', name: 'Madrid', price: 60, tax: 40, color: 'brown', id: 22 },
  { type: 'chance' },
  { type: 'destination', name: 'Giethoorn', price: 60, tax: 40, color: 'brown', id: 21 },
  { type: 'airport' },
  { type: 'here_and_now' },
  { type: 'destination', name: 'Taipei', price: 100, tax: 60, color: 'lightblue', id: 20 },
  { type: 'first_class' },
  { type: 'destination', name: 'Cape Town', price: 100, tax: 60, color: 'lightblue', id: 19 },
  { type: 'destination', name: 'Queenstown', price: 100, tax: 60, color: 'lightblue', id: 18 },
  { type: 'just_visiting' },
  { type: 'destination', name: 'Sydney', price: 160, tax: 100, color: 'pink', id: 17 },
  { type: 'chance' },
  { type: 'destination', name: 'Amsterdam', price: 160, tax: 100, color: 'pink', id: 16 },
  { type: 'destination', name: 'New York', price: 160, tax: 100, color: 'pink', id: 15 },
  { type: 'here_and_now' },
  { type: 'destination', name: 'Tokyo', price: 200, tax: 120, color: 'orange', id: 14 },
  { type: 'first_class' },
  { type: 'destination', name: 'Moscow', price: 200, tax: 120, color: 'orange', id: 13 },
  { type: 'destination', name: 'London', price: 200, tax: 120, color: 'orange', id: 12 },
  { type: 'free_parking' },
  { type: 'destination', name: 'Belgrade', price: 260, tax: 140, color: 'red', id: 11 },
  { type: 'chance' },
  { type: 'destination', name: 'Athens', price: 260, tax: 140, color: 'red', id: 10 },
  { type: 'destination', name: 'Belfast', price: 260, tax: 140, color: 'red', id: 9 },
  { type: 'here_and_now' },
  { type: 'destination', name: 'Santiago', price: 300, tax: 180, color: 'yellow', id: 8 },
  { type: 'destination', name: 'Mexico City', price: 300, tax: 180, color: 'yellow', id: 7 },
  { type: 'first_class' },
  { type: 'destination', name: 'Warsaw', price: 300, tax: 180, color: 'yellow', id: 6 },
  { type: 'go_to_jail' },
  { type: 'destination', name: 'Istanbul', price: 360, tax: 200, color: 'green', id: 5 },
  { type: 'destination', name: 'Lisbon', price: 360, tax: 200, color: 'green', id: 4 },
  { type: 'chance' },
  { type: 'destination', name: 'Riga', price: 360, tax: 200, color: 'green', id: 3 },
  { type: 'here_and_now' },
  { type: 'airport' },
  { type: 'destination', name: 'Hong Kong', price: 400, tax: 240, color: 'darkblue', id: 2 },
  { type: 'first_class' },
  { type: 'destination', name: 'Lima', price: 400, tax: 240, color: 'darkblue', id: 1 },
]

// Separate spaces by position on the board
const bottomRow = computed(() => boardSpaces.slice(1, 10).reverse())
const leftColumn = computed(() => boardSpaces.slice(11, 20))
const topRow = computed(() => boardSpaces.slice(21, 30))
const rightColumn = computed(() => boardSpaces.slice(31, 40))

function getColorStyle(color?: keyof typeof COLORS): string {
  if (!color) return ''
  return COLORS[color]
}

function getSpaceIcon(type: string): string {
  switch (type) {
    case 'chance': return '?'
    case 'airport': return '✈'
    case 'here_and_now': return '⭐'
    case 'first_class': return '💎'
    default: return ''
  }
}
</script>

<template>
  <div class="board-container">
    <div class="board">
      <!-- Top row -->
      <div class="board-row top-row">
        <!-- FREE PARKING corner -->
        <div class="corner-space">
          <div class="corner-content free-parking">
            <div class="corner-icon">🅿️</div>
            <span class="corner-label">FREE<br>PARKING</span>
          </div>
        </div>
        
        <div class="spaces-row">
          <div 
            v-for="(space, index) in topRow" 
            :key="'top-' + index"
            class="space"
            :class="space.type"
          >
            <div 
              v-if="space.color" 
              class="color-band"
              :style="{ background: getColorStyle(space.color) }"
            ></div>
            <div class="space-content">
              <template v-if="space.type === 'destination'">
                <span class="space-name">{{ space.name }}</span>
                <span class="space-price">M{{ space.price }}</span>
              </template>
              <template v-else>
                <span class="space-icon">{{ getSpaceIcon(space.type) }}</span>
                <span class="space-type-label">{{ space.type.replace(/_/g, ' ').toUpperCase() }}</span>
              </template>
            </div>
            
            <!-- Property Stamp -->
            <div v-if="space.color" class="property-stamp">
              <Stamp 
                :color-type="space.color === 'darkblue' ? 'blue' : space.color"
                :number="space.id"
                :label="space.name"
              />
            </div>

          </div>
        </div>
        
        <!-- GO TO JAIL corner -->
        <div class="corner-space">
          <div class="corner-content go-to-jail">
            <div class="corner-icon">🚔</div>
            <span class="corner-label">GO TO<br>JAIL</span>
          </div>
        </div>
      </div>
      
      <!-- Middle section -->
      <div class="board-middle">
        <!-- Left column -->
        <div class="spaces-column left-column">
          <div 
            v-for="(space, index) in leftColumn" 
            :key="'left-' + index"
            class="space horizontal-space left-side"
            :class="space.type"
          >
            <div 
              v-if="space.color" 
              class="color-band"
              :style="{ background: getColorStyle(space.color) }"
            ></div>
            <div class="space-content">
              <template v-if="space.type === 'destination'">
                <span class="space-name">{{ space.name }}</span>
                <span class="space-price">M{{ space.price }}</span>
              </template>
              <template v-else>
                <span class="space-icon">{{ getSpaceIcon(space.type) }}</span>
                <span class="space-type-label">{{ space.type.replace(/_/g, ' ').toUpperCase() }}</span>
              </template>
            </div>
            <!-- Property Stamp -->
            <div v-if="space.color" class="property-stamp">
              <Stamp 
                :color-type="space.color === 'darkblue' ? 'blue' : space.color"
                :number="space.id"
                :label="space.name"
              />
            </div>

          </div>
        </div>
        
        <!-- Center area with 4 passport zones in corners -->
        <div class="board-center">
          <!-- Passport zone TOP-LEFT -->
          <div class="passport-zone zone-top-left">
            <div class="passport-area">
              <Passport />
            </div>
          </div>
          
          <!-- Passport zone TOP-RIGHT -->
          <div class="passport-zone zone-top-right">
            <div class="passport-area">
              <Passport />
            </div>
          </div>
          
          <!-- Center Logo -->
          <div class="center-logo">
            <div class="globe-container">
              <div class="globe"></div>
            </div>
            <h1 class="logo-text">MONOPOLY</h1>
            <h2 class="edition-text">WORLD EDITION</h2>
          </div>
          
          <!-- Passport zone BOTTOM-LEFT -->
          <div class="passport-zone zone-bottom-left">
            <div class="passport-area">
              <Passport />
            </div>
          </div>
          
          <!-- Passport zone BOTTOM-RIGHT -->
          <div class="passport-zone zone-bottom-right">
            <div class="passport-area">
              <Passport />
            </div>
          </div>
        </div>
        
        <!-- Right column -->
        <div class="spaces-column right-column">
          <div 
            v-for="(space, index) in rightColumn" 
            :key="'right-' + index"
            class="space horizontal-space right-side"
            :class="space.type"
          >
            <div 
              v-if="space.color" 
              class="color-band"
              :style="{ background: getColorStyle(space.color) }"
            ></div>
            <div class="space-content">
              <template v-if="space.type === 'destination'">
                <span class="space-name">{{ space.name }}</span>
                <span class="space-price">M{{ space.price }}</span>
              </template>
              <template v-else>
                <span class="space-icon">{{ getSpaceIcon(space.type) }}</span>
                <span class="space-type-label">{{ space.type.replace(/_/g, ' ').toUpperCase() }}</span>
              </template>
            </div>
            <!-- Property Stamp -->
            <div v-if="space.color" class="property-stamp">
              <Stamp 
                :color-type="space.color === 'darkblue' ? 'blue' : space.color"
                :number="space.id"
                :label="space.name"
              />
            </div>

          </div>
        </div>
      </div>
      
      <!-- Bottom row -->
      <div class="board-row bottom-row">
        <!-- JUST VISITING corner -->
        <div class="corner-space">
          <div class="corner-content just-visiting">
            <div class="jail-box">
              <span>JAIL</span>
            </div>
            <span class="corner-label visiting-label">JUST<br>VISITING</span>
          </div>
        </div>
        
        <div class="spaces-row">
          <div 
            v-for="(space, index) in bottomRow" 
            :key="'bottom-' + index"
            class="space bottom-space"
            :class="space.type"
          >
            <div 
              v-if="space.color" 
              class="color-band"
              :style="{ background: getColorStyle(space.color) }"
            ></div>
            <div class="space-content">
              <template v-if="space.type === 'destination'">
                <span class="space-name">{{ space.name }}</span>
                <span class="space-price">M{{ space.price }}</span>
              </template>
              <template v-else>
                <span class="space-icon">{{ getSpaceIcon(space.type) }}</span>
                <span class="space-type-label">{{ space.type.replace(/_/g, ' ').toUpperCase() }}</span>
              </template>
            </div>
            <!-- Property Stamp -->
            <div v-if="space.color" class="property-stamp">
              <Stamp 
                :color-type="space.color === 'darkblue' ? 'blue' : space.color"
                :number="space.id"
                :label="space.name"
              />
            </div>

          </div>
        </div>
        
        <!-- START corner -->
        <div class="corner-space">
          <div class="corner-content start">
            <div class="start-arrow">➡️</div>
            <span class="corner-label start-label">START</span>
            <span class="start-bonus">Collect M200</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
@import url('https://fonts.googleapis.com/css2?family=Oswald:wght@400;500;600;700&family=Roboto:wght@400;500;700&display=swap');

.board-container {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 20px;
  min-height: 100vh;
  background: linear-gradient(135deg, #0a1628 0%, #0d2137 50%, #0a1628 100%);
}

.board {
  --board-size: min(92vw, 92vh, 850px);
  --corner-size: calc(var(--board-size) * 0.115);
  --space-width: calc((var(--board-size) - 2 * var(--corner-size)) / 9);
  --space-height: var(--corner-size);
  
  width: var(--board-size);
  height: var(--board-size);
  position: relative;
  /* Blue gradient background like the physical board */
  background: 
    radial-gradient(ellipse at 30% 30%, rgba(100, 180, 255, 0.15) 0%, transparent 50%),
    radial-gradient(ellipse at 70% 70%, rgba(50, 150, 220, 0.1) 0%, transparent 40%),
    linear-gradient(180deg, 
      #1565C0 0%, 
      #1E88E5 15%,
      #42A5F5 30%,
      #64B5F6 45%,
      #90CAF9 60%,
      #BBDEFB 75%,
      #E3F2FD 90%,
      #f0f8ff 100%
    );
  border: 5px solid #0D47A1;
  border-radius: 6px;
  box-shadow: 
    0 0 0 10px #1565C0,
    0 0 0 12px #0D47A1,
    0 25px 80px rgba(0, 0, 0, 0.6),
    inset 0 0 60px rgba(255, 255, 255, 0.05);
  overflow: hidden;
}

/* Rows and columns */
.board-row {
  display: flex;
  position: absolute;
  left: 0;
  right: 0;
}

.top-row {
  top: 0;
  height: var(--corner-size);
}

.bottom-row {
  bottom: 0;
  height: var(--corner-size);
}

.board-middle {
  position: absolute;
  top: var(--corner-size);
  bottom: var(--corner-size);
  left: 0;
  right: 0;
  display: flex;
}

.spaces-row {
  display: flex;
  flex: 1;
}

.spaces-column {
  display: flex;
  flex-direction: column;
  width: var(--corner-size);
}

.left-column {
  flex-direction: column-reverse;
}

/* Corner spaces - consistent sizing */
.corner-space {
  width: var(--corner-size);
  height: var(--corner-size);
  background: linear-gradient(135deg, #f8f9fa 0%, #e9ecef 100%);
  border: 2px solid #0D47A1;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  box-sizing: border-box;
}

.corner-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  width: 100%;
  height: 100%;
  padding: 4px;
  box-sizing: border-box;
}

.corner-icon {
  font-size: calc(var(--corner-size) * 0.28);
  line-height: 1;
}

.corner-label {
  font-family: 'Oswald', sans-serif;
  font-size: calc(var(--corner-size) * 0.11);
  font-weight: 700;
  color: #0D47A1;
  line-height: 1.1;
  text-transform: uppercase;
  margin-top: 2px;
}

/* START corner */
.corner-content.start {
  background: linear-gradient(135deg, #ff6b6b 0%, #ee5a5a 100%);
}

.start-arrow {
  font-size: calc(var(--corner-size) * 0.32);
  transform: rotate(-45deg);
  line-height: 1;
}

.start-label {
  color: white;
  font-size: calc(var(--corner-size) * 0.16);
  text-shadow: 1px 1px 2px rgba(0,0,0,0.3);
}

.start-bonus {
  font-family: 'Roboto', sans-serif;
  font-size: calc(var(--corner-size) * 0.08);
  color: white;
  margin-top: 2px;
}

/* JUST VISITING corner */
.corner-content.just-visiting {
  background: linear-gradient(135deg, #f8f9fa 0%, #e9ecef 100%);
  position: relative;
}

.jail-box {
  position: absolute;
  top: 6px;
  left: 6px;
  width: calc(var(--corner-size) * 0.42);
  height: calc(var(--corner-size) * 0.42);
  background: #ff9f43;
  border: 2px solid #0D47A1;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: 'Oswald', sans-serif;
  font-size: calc(var(--corner-size) * 0.11);
  font-weight: 700;
  color: #0D47A1;
  box-sizing: border-box;
}

.visiting-label {
  position: absolute;
  bottom: 3px;
  right: 3px;
  font-size: calc(var(--corner-size) * 0.09);
  text-align: right;
}

/* FREE PARKING corner */
.corner-content.free-parking {
  background: linear-gradient(135deg, #a8e6cf 0%, #88d8b0 100%);
}

/* GO TO JAIL corner */
.corner-content.go-to-jail {
  background: linear-gradient(135deg, #ffd93d 0%, #ff9f43 100%);
}

/* Regular spaces */
.space {
  width: var(--space-width);
  height: var(--corner-size);
  background: linear-gradient(180deg, #f8f9fa 0%, #e9ecef 100%);
  border: 1px solid #0D47A1;
  display: flex;
  flex-direction: column;
  position: relative;
  box-sizing: border-box;
}

.horizontal-space {
  width: var(--corner-size);
  height: var(--space-width);
  flex-direction: row;
}

.horizontal-space .color-band {
  width: 22%;
  height: 100%;
}

.left-side .color-band {
  order: 1;
}

.right-side .color-band {
  order: -1;
}

.bottom-space {
  flex-direction: column-reverse;
}

.bottom-space .color-band {
  order: 1;
}

/* Color bands */
.color-band {
  width: 100%;
  height: 24%;
  flex-shrink: 0;
}

/* Space content */
.space-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 2px;
  text-align: center;
}

.horizontal-space .space-content {
  writing-mode: vertical-rl;
  text-orientation: mixed;
}

/* Left side spaces: Text should face left (outwards) - Default vertical-rl has base to left */
.left-side .space-content {
  transform: none;
}

/* Right side spaces: Text should face right (outwards) - Needs 180 rotation from base-left */
.right-side .space-content {
  transform: rotate(180deg);
}

.top-row .space {
  flex-direction: column-reverse; /* Color band at bottom (interior) */
}

.top-row .space-content {
  transform: rotate(180deg);
}

/* Property Stamp Overlay - perfectly centered on the color bar */
.property-stamp {
  position: absolute;
  z-index: 10;
  pointer-events: none;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* Positioning for vertical fields (top/bottom) */
/* Centered on the border with a slight 6px interior offset logic */
.top-row .property-stamp {
  bottom: 6px; /* Anchor 6px inside, translate 50% down (towards center) */
  left: 50%;
  transform: translate(-50%, 50%) scale(0.24) rotate(180deg);
}
.bottom-row .property-stamp {
  top: 6px; /* Anchor 6px inside, translate 50% up (towards center) */
  left: 50%;
  transform: translate(-50%, -50%) scale(0.24);
}

/* Positioning for horizontal fields (left/right) */
/* Aligned to the interior edge to allow border overlap */
.left-column .property-stamp {
  right: 6px;
  top: 52%;
  transform: translate(50%, -50%) scale(0.24) rotate(90deg);
}
.right-column .property-stamp {
  left: 6px;
  top: 52%;
  transform: translate(-50%, -50%) scale(0.24) rotate(-90deg);
}

.space-name {
  font-family: 'Oswald', sans-serif;
  font-size: calc(var(--space-width) * 0.14);
  font-weight: 600;
  color: #0D47A1;
  line-height: 1.1;
  text-transform: uppercase;
}

.space-price {
  font-family: 'Roboto', sans-serif;
  font-size: calc(var(--space-width) * 0.11);
  color: #1565C0;
  font-weight: 500;
}



/* Special space types */
.space.chance {
  background: linear-gradient(135deg, #e3f2fd 0%, #bbdefb 100%);
}

.space.airport {
  background: linear-gradient(135deg, #fff8e1 0%, #ffecb3 100%);
}

.space.here_and_now {
  background: linear-gradient(135deg, #e8f5e9 0%, #c8e6c9 100%);
}

.space.first_class {
  background: linear-gradient(135deg, #f3e5f5 0%, #e1bee7 100%);
}

.space-icon {
  font-size: calc(var(--space-width) * 0.22);
  margin-bottom: 2px;
}

.space-type-label {
  font-family: 'Oswald', sans-serif;
  font-size: calc(var(--space-width) * 0.085);
  font-weight: 500;
  color: #37474f;
  text-transform: uppercase;
}

/* Board center - Rigid container for passports */
.board-center {
  flex: 1;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden; /* CRITICAL: Clips anything that tries to overlap towns */
  background: transparent;
}

/* Passport zones in 4 corners of center - No background, rotated */
.passport-zone {
  position: absolute;
  width: 0;
  height: 0;
  z-index: 5;
}

.zone-top-left {
  top: 5px;
  left: 40px;
}
.zone-top-left .passport-area {
  position: absolute;
  bottom: 0;
  left: 0;
  transform: scale(0.26) rotate(135deg);
  transform-origin: bottom left;
}

.zone-top-right {
  top: 5px;
  right: 40px;
}
.zone-top-right .passport-area {
  position: absolute;
  bottom: 0;
  right: 0;
  transform: scale(0.26) rotate(-135deg);
  transform-origin: bottom right;
}

.zone-bottom-left {
  bottom: 40px;
  left: 1px;
}
.zone-bottom-left .passport-area {
  position: absolute;
  bottom: 0;
  left: 0;
  transform: scale(0.26) rotate(45deg);
  transform-origin: bottom left;
}

.zone-bottom-right {
  bottom: 40px;
  right: 1px;
}
.zone-bottom-right .passport-area {
  position: absolute;
  bottom: 0;
  right: 0;
  transform: scale(0.26) rotate(-45deg);
  transform-origin: bottom right;
}

.passport-area {
  box-shadow: 0 10px 25px rgba(0,0,0,0.4);
}

/* Logo in center */
.center-logo {
  display: flex;
  flex-direction: column;
  align-items: center;
  z-index: 10;
}

.globe-container {
  width: calc((var(--board-size) - 2 * var(--corner-size)) * 0.12);
  height: calc((var(--board-size) - 2 * var(--corner-size)) * 0.12);
  margin-bottom: 8px;
}

.globe {
  width: 100%;
  height: 100%;
  border-radius: 50%;
  background: linear-gradient(135deg, #4fc3f7 0%, #0288d1 50%, #01579b 100%);
  box-shadow: 
    inset -8px -8px 16px rgba(0,0,0,0.3),
    inset 8px 8px 16px rgba(255,255,255,0.2),
    0 4px 12px rgba(0,0,0,0.3);
  position: relative;
}

.globe::before {
  content: '';
  position: absolute;
  inset: 12%;
  border-radius: 50%;
  border: 2px dashed rgba(255,255,255,0.35);
}

.globe::after {
  content: '';
  position: absolute;
  width: 110%;
  height: 2px;
  background: rgba(255,255,255,0.35);
  top: 50%;
  left: -5%;
  transform: translateY(-50%);
}

.logo-text {
  font-family: 'Oswald', sans-serif;
  font-size: calc((var(--board-size) - 2 * var(--corner-size)) * 0.065);
  font-weight: 700;
  color: #c41e3a;
  text-shadow: 
    2px 2px 0 #fff,
    -1px -1px 0 #fff,
    1px -1px 0 #fff,
    -1px 1px 0 #fff;
  margin: 0;
  letter-spacing: 3px;
}

.edition-text {
  font-family: 'Oswald', sans-serif;
  font-size: calc((var(--board-size) - 2 * var(--corner-size)) * 0.028);
  font-weight: 500;
  color: #0D47A1;
  margin: 0;
  letter-spacing: 4px;
  text-transform: uppercase;
}

/* Responsive adjustments */
@media (max-width: 600px) {
  .space-name {
    font-size: calc(var(--space-width) * 0.12);
  }
  
  .space-price {
    font-size: calc(var(--space-width) * 0.1);
  }
  
  .corner-label {
    font-size: calc(var(--corner-size) * 0.09);
  }
  
  .passport-area {
    transform: scale(0.25);
  }
}
</style>
