<script setup lang="ts">
import { computed } from 'vue'
import Passport from './Passport.vue'
import Stamp from './Stamp.vue'
import GameDice from './GameDice.vue'
import CardStack from './CardStack.vue'
import GameToken from './GameToken.vue'

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
                <div class="special-content-wrapper">
                  <!-- First Class -->
                  <div v-if="space.type === 'first_class'" class="first-class-content">
                    <span class="special-label">FIRST CLASS</span>
                    <div class="elegant-star-ring-v2">
                      <div class="elegant-star-v2">★</div>
                    </div>
                    <span class="special-price">M100</span>
                  </div>
                  
                  <!-- Airport -->
                  <div v-else-if="space.type === 'airport'" class="airport-content">
                    <span class="special-label">AIRPORT</span>
                    <div class="airport-emoji">✈️</div>
                    <span class="special-price">M100</span>
                  </div>

                  <!-- Here & Now (Preserved) -->
                  <div v-else-if="space.type === 'here_and_now'" class="here-now-content">
                    <div class="globe-motif">
                      <div class="globe-icon-outline">🌎</div>
                    </div>
                    <div class="here-now-text">
                      <span class="text-here">HERE</span>
                      <span class="text-and">&</span>
                      <span class="text-now">NOW</span>
                    </div>
                  </div>

                  <!-- Chance (Preserved) -->
                  <div v-else-if="space.type === 'chance'" class="chance-content">
                    <span class="large-question-mark">?</span>
                    <span class="chance-label">CHANCE</span>
                  </div>

                  <!-- Other special spaces -->
                  <template v-else>
                    <span class="space-icon">{{ getSpaceIcon(space.type) }}</span>
                    <span class="space-type-label">{{ space.type.replace(/_/g, ' ').toUpperCase() }}</span>
                  </template>
                </div>
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
                <div class="special-content-wrapper">
                  <!-- First Class -->
                  <div v-if="space.type === 'first_class'" class="first-class-content">
                    <span class="special-label">FIRST CLASS</span>
                    <div class="elegant-star-ring-v2">
                      <div class="elegant-star-v2">★</div>
                    </div>
                    <span class="special-price">M100</span>
                  </div>
                  
                  <!-- Airport -->
                  <div v-else-if="space.type === 'airport'" class="airport-content">
                    <span class="special-label">AIRPORT</span>
                    <div class="airport-emoji">✈️</div>
                    <span class="special-price">M100</span>
                  </div>

                  <!-- Here & Now (Preserved) -->
                  <div v-else-if="space.type === 'here_and_now'" class="here-now-content">
                    <div class="globe-motif">
                      <div class="globe-icon-outline">🌎</div>
                    </div>
                    <div class="here-now-text">
                      <span class="text-here">HERE</span>
                      <span class="text-and">&</span>
                      <span class="text-now">NOW</span>
                    </div>
                  </div>

                  <!-- Chance (Preserved) -->
                  <div v-else-if="space.type === 'chance'" class="chance-content">
                    <span class="large-question-mark">?</span>
                    <span class="chance-label">CHANCE</span>
                  </div>

                  <!-- Other special spaces -->
                  <template v-else>
                    <span class="space-icon">{{ getSpaceIcon(space.type) }}</span>
                    <span class="space-type-label">{{ space.type.replace(/_/g, ' ').toUpperCase() }}</span>
                  </template>
                </div>
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
          
          <!-- Deck Placeholders -->
          <!-- Deck Placeholders -->
          <div class="card-deck chance-deck">
            <CardStack type="chance" />
          </div>
          <div class="card-deck here-now-deck">
            <CardStack type="here_and_now" />
          </div>

          <!-- Dice in the center -->
          <div class="dice-container-center">
            <GameDice :forced-deal="true" />
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
                <div class="special-content-wrapper">
                  <!-- First Class -->
                  <div v-if="space.type === 'first_class'" class="first-class-content">
                    <span class="special-label">FIRST CLASS</span>
                    <div class="elegant-star-ring-v2">
                      <div class="elegant-star-v2">★</div>
                    </div>
                    <span class="special-price">M100</span>
                  </div>
                  
                  <!-- Airport -->
                  <div v-else-if="space.type === 'airport'" class="airport-content">
                    <span class="special-label">AIRPORT</span>
                    <div class="airport-emoji">✈️</div>
                    <span class="special-price">M100</span>
                  </div>

                  <!-- Here & Now (Preserved) -->
                  <div v-else-if="space.type === 'here_and_now'" class="here-now-content">
                    <div class="globe-motif">
                      <div class="globe-icon-outline">🌎</div>
                    </div>
                    <div class="here-now-text">
                      <span class="text-here">HERE</span>
                      <span class="text-and">&</span>
                      <span class="text-now">NOW</span>
                    </div>
                  </div>

                  <!-- Chance (Preserved) -->
                  <div v-else-if="space.type === 'chance'" class="chance-content">
                    <span class="large-question-mark">?</span>
                    <span class="chance-label">CHANCE</span>
                  </div>

                  <!-- Other special spaces -->
                  <template v-else>
                    <span class="space-icon">{{ getSpaceIcon(space.type) }}</span>
                    <span class="space-type-label">{{ space.type.replace(/_/g, ' ').toUpperCase() }}</span>
                  </template>
                </div>
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
                <div class="special-content-wrapper">
                  <!-- First Class -->
                  <div v-if="space.type === 'first_class'" class="first-class-content">
                    <span class="special-label">FIRST CLASS</span>
                    <div class="elegant-star-ring-v2">
                      <div class="elegant-star-v2">★</div>
                    </div>
                    <span class="special-price">M100</span>
                  </div>
                  
                  <!-- Airport -->
                  <div v-else-if="space.type === 'airport'" class="airport-content">
                    <span class="special-label">AIRPORT</span>
                    <div class="airport-emoji">✈️</div>
                    <span class="special-price">M100</span>
                  </div>

                  <!-- Here & Now (Preserved) -->
                  <div v-else-if="space.type === 'here_and_now'" class="here-now-content">
                    <div class="globe-motif">
                      <div class="globe-icon-outline">🌎</div>
                    </div>
                    <div class="here-now-text">
                      <span class="text-here">HERE</span>
                      <span class="text-and">&</span>
                      <span class="text-now">NOW</span>
                    </div>
                  </div>

                  <!-- Chance (Preserved) -->
                  <div v-else-if="space.type === 'chance'" class="chance-content">
                    <span class="large-question-mark">?</span>
                    <span class="chance-label">CHANCE</span>
                  </div>

                  <!-- Other special spaces -->
                  <template v-else>
                    <span class="space-icon">{{ getSpaceIcon(space.type) }}</span>
                    <span class="space-type-label">{{ space.type.replace(/_/g, ' ').toUpperCase() }}</span>
                  </template>
                </div>
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
            <div class="start-diamond-final">
              <div class="start-arrow-final">➜</div>
            </div>
            <span class="corner-label start-label">START</span>
            <span class="start-bonus">Collect M200</span>
            
            <!-- Game Tokens at Start -->
            <div class="tokens-at-start">
              <GameToken type="seal" />
              <GameToken type="cat" />
              <GameToken type="capybara" />
              <GameToken type="dog" />
            </div>
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
    radial-gradient(ellipse at 30% 30%, rgba(38, 89, 255, 0.15) 0%, transparent 50%),
    radial-gradient(ellipse at 70% 70%, rgba(38, 89, 255, 0.1) 0%, transparent 40%),
    linear-gradient(180deg, 
      #000000 0%, 
      #050a14 15%,
      #0a1428 30%,
      #0f1e3c 45%,
      #142850 60%,
      #193264 75%,
      #1e3c78 90%,
      #2659FF 100%
    );
  border: 5px solid #000000;
  border-radius: 6px;
  box-shadow: 
    0 0 0 10px #000000,
    0 0 0 12px #101010,
    0 25px 80px rgba(0, 0, 0, 0.8),
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
  border: 2px solid #000000;
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

.start-diamond-final {
  width: calc(var(--corner-size) * 0.3);
  height: calc(var(--corner-size) * 0.3);
  background: #1976D2;
  transform: rotate(45deg);
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 4px 10px rgba(0,0,0,0.25);
  margin-bottom: 8px;
  border-radius: 6px;
  border: 2px solid white;
}

.start-arrow-final {
  font-size: calc(var(--corner-size) * 0.2);
  color: white;
  transform: rotate(135deg); /* Points Left */
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

.tokens-at-start {
  position: absolute;
  bottom: 8px;
  right: 8px;
  display: flex;
  flex-wrap: wrap;
  width: 50px;
  gap: 4px;
  pointer-events: none;
  z-index: 25;
}

.tokens-at-start > * {
  pointer-events: auto;
}

/* JUST VISITING corner */
.corner-content.just-visiting {
  background: #000000;
  position: relative;
}

.jail-box {
  position: absolute;
  top: 4px;
  left: 4px;
  width: calc(var(--corner-size) * 0.58);
  height: calc(var(--corner-size) * 0.58);
  background: #C44601;
  border: 3px solid #000000;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: 'Oswald', sans-serif;
  font-size: calc(var(--corner-size) * 0.13);
  font-weight: 700;
  color: white;
  box-sizing: border-box;
  box-shadow: inset 0 0 15px rgba(0,0,0,0.4);
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
  border: 1px solid #000000;
  display: flex;
  flex-direction: column;
  position: relative;
  box-sizing: border-box;
}

.space.destination {
  background: linear-gradient(0deg, #000000 0%, #0077FF 100%);
}

/* Side-specific property gradients: Outer Black -> Inner Blue */
.bottom-row .space.destination {
  background: linear-gradient(0deg, #000000 0%, #0077FF 100%);
}

.top-row .space.destination {
  background: linear-gradient(180deg, #000000 0%, #0077FF 100%);
}

.left-column .space.destination {
  background: linear-gradient(90deg, #000000 0%, #0077FF 100%);
}

.right-column .space.destination {
  background: linear-gradient(270deg, #000000 0%, #0077FF 100%);
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
  position: relative; /* Needed for absolute special wrappers */
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
  color: white;
  line-height: 1.1;
  text-transform: uppercase;
}

.space-price {
  font-family: 'Roboto', sans-serif;
  font-size: calc(var(--space-width) * 0.11);
  color: #a0c0ff;
  font-weight: 500;
}

.space:not(.destination) .space-name {
  color: #0D47A1;
}

.space:not(.destination) .space-price {
  color: #1565C0;
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

/* Special Space Redesigns */
.space.first_class {
  background: radial-gradient(circle at center, #f5f5f5 0%, #e0e0e0 100%);
}

/* Standardized Special Content Wrapper */
.special-content-wrapper {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  position: relative;
  writing-mode: horizontal-tb !important;
  overflow: visible; /* Ensure nothing is cut off */
}

/* Specific rotations for left/right special spaces content - use square wrapping to avoid clipping */
.left-side .special-content-wrapper,
.right-side .special-content-wrapper {
  position: absolute;
  top: 50%;
  left: 50%;
  width: var(--corner-size);
  height: var(--corner-size);
  z-index: 5;
}

.left-side .special-content-wrapper {
  transform: translate(-50%, -50%) rotate(90deg);
}

.right-side .special-content-wrapper {
  transform: translate(-50%, -50%) rotate(90deg);
}

.first-class-content, .airport-content, .here-now-content, .chance-content {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 4px;
  box-sizing: border-box;
  gap: 2px;
}

.special-label {
  font-family: 'Oswald', sans-serif;
  font-size: calc(var(--space-width) * 0.09);
  font-weight: 700;
  color: #333;
  text-transform: uppercase;
}

.special-price {
  font-family: 'Roboto', sans-serif;
  font-size: calc(var(--space-width) * 0.08);
  font-weight: 700;
  color: #333;
}

/* Elegant First Class Logo v2 */
.elegant-star-ring-v2 {
  width: calc(var(--space-width) * 0.5);
  height: calc(var(--space-width) * 0.5);
  border: 1.5px solid #FFD700;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: white;
  position: relative;
  box-shadow: 0 0 8px rgba(255, 215, 0, 0.2);
}

.elegant-star-ring-v2::after {
  content: '';
  position: absolute;
  inset: 2px;
  border: 1px solid #FFD54F;
  border-radius: 50%;
}

.elegant-star-v2 {
  font-size: calc(var(--space-width) * 0.3);
  color: #FFD700;
  line-height: 1;
  z-index: 1;
}

.space.first_class {
  background: radial-gradient(circle at center, #ffffff 0%, #e0e0e0 100%);
}

/* Airport simple icon */
.space.airport {
  background: #03A9F4;
}

.space.airport .special-label, .space.airport .special-price {
  color: white;
}

.airport-emoji {
  font-size: calc(var(--space-width) * 0.4);
  filter: drop-shadow(0 0 2px rgba(0,0,0,0.2));
}

/* Here & Now (Preserved) */
.space.here_and_now {
  background: #cc0000;
}

.globe-icon-outline {
  font-size: calc(var(--space-width) * 0.35);
}

.here-now-text {
  display: flex;
  flex-direction: column;
  align-items: center;
  line-height: 1;
}

.here-now-text span {
  font-family: 'Oswald', sans-serif;
  color: white;
  font-weight: 700;
  text-transform: uppercase;
}

.text-here { font-size: calc(var(--space-width) * 0.14); }
.text-and { font-size: calc(var(--space-width) * 0.1); color: #ddd; }
.text-now { font-size: calc(var(--space-width) * 0.18); }

/* Chance (Preserved) */
.space.chance {
  background: #9c27b0;
}

.large-question-mark {
  font-family: 'Oswald', sans-serif;
  font-size: calc(var(--space-width) * 0.6);
  color: white;
  font-weight: 700;
  line-height: 1;
}

.chance-label {
  font-family: 'Oswald', sans-serif;
  font-size: calc(var(--space-width) * 0.1);
  color: white;
  font-weight: 400;
  letter-spacing: 1px;
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

/* Decks in center */
.card-deck {
  position: absolute;
  width: calc(var(--board-size) * 0.12);
  height: calc(var(--board-size) * 0.18);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2;
}

.chance-deck {
  top: 50%;
  left: 18%;
  transform: translate(-50%, -50%);
}

.here-now-deck {
  top: 50%;
  right: 18%;
  transform: translate(50%, -50%);
}

.deck-outline {
  width: 100%;
  height: 100%;
  border: 2px dashed #ff5252;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent; /* Cleaner without background */
}

.dice-container-center {
  z-index: 10;
  display: flex;
  justify-content: center;
  align-items: center;
  pointer-events: none; /* Allow clicks through if needed */
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
