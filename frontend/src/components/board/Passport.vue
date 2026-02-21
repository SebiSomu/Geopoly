<script setup lang="ts">

import { ref, computed } from 'vue';
import Stamp from './Stamp.vue';
import GameToken from './GameToken.vue';
import type { StampColor } from './Stamp.vue';

// ============ Type Definitions ============
export interface PassportProperty {
  name: string;
  color: string;
  diameter: number;
  column: 'left' | 'right';
  destination_id?: number | null;
}

export interface PlacedStamp {
  id: string;
  colorType: StampColor;
  number: number | string;
  x: number;
  y: number;
  rotation: number;
  column: 'left' | 'right';
  size: number;
}

const props = defineProps<{
  properties?: PassportProperty[];
  playerName?: string;
  character?: 'seal' | 'capybara' | 'cat' | 'dog';
}>();

// ============ State ============
const localStamps = ref<PlacedStamp[]>([]); // Fallback for drag & drop or local testing
const leftColumnRef = ref<HTMLElement | null>(null);
const rightColumnRef = ref<HTMLElement | null>(null);

// ============ Drag & Drop Handlers ============
const onDragOver = (event: DragEvent): void => {
  event.preventDefault();
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'copy';
  }
};

const onDrop = (event: DragEvent, column: 'left' | 'right'): void => {
  event.preventDefault();
  const data = event.dataTransfer?.getData('application/json');
  if (!data) return;

  const columnRef = column === 'left' ? leftColumnRef.value : rightColumnRef.value;
  if (!columnRef) return;

  try {
    const droppedStamp = JSON.parse(data);
    const columnRect = columnRef.getBoundingClientRect();

    // Center stamp on drop position
    const x = event.clientX - columnRect.left - (droppedStamp.size || 35);
    // Align y to bottom-relative for consistency with sync logic
    const y = columnRect.bottom - event.clientY - (droppedStamp.size || 35);
    const rotation = Math.floor(Math.random() * 16) - 8;

    let colorType = droppedStamp.colorType.toLowerCase();
    if (colorType === 'gray') colorType = 'grey';
    if (colorType === 'darkblue') colorType = 'blue';

    const stampSize = SIZES[colorType as keyof typeof SIZES] || 35;

    localStamps.value.push({
      id: `stamp-${Date.now()}-${Math.random().toString(36).substr(2, 5)}`,
      colorType: colorType as any,
      number: droppedStamp.number,
      x,
      y,
      rotation,
      column,
      size: stampSize,
    });
  } catch (e) {
    console.error('Failed to parse dropped stamp data:', e);
  }
};

// ============ Clear Stamps ============
const clearStamps = (): void => {
  localStamps.value = [];
};

// Local mapping of sizes for positioning (copied from Stamp.vue to ensure 1:1 match)
const SIZES = {
  grey: 56, 
  brown: 60, 
  lightblue: 64, 
  pink: 72, 
  orange: 74, 
  red: 84, 
  yellow: 88, 
  green: 98, 
  blue: 100,
};

const processedStamps = computed(() => {
  const result: PlacedStamp[] = [...localStamps.value];
  
  if (props.properties) {
    let leftUsedHeight = 0;
    let rightUsedHeight = 0;
    let leftCount = 0;
    let rightCount = 0;
    
    props.properties.forEach((p, index) => {
      // Normalize color name
      let colorKey = p.color.toLowerCase();
      if (colorKey === 'gray') colorKey = 'grey';
      if (colorKey === 'darkblue') colorKey = 'blue';

      // Priority: use the diameter sent from server, fallback to hardcoded map
      const diameterPx = Math.round(p.diameter * 40) || SIZES[colorKey as keyof typeof SIZES] || 35;
      const rotation = (index * 7) % 15 - 7;
      
      let x = 0;
      let y = 0;
      
      // Calculate available space for stagger to prevent bleeding outside 100px column
      const columnWidth = 100;
      const centerX = (columnWidth - diameterPx) / 2;
      const allowedStagger = Math.max(0, Math.min(12, centerX)); 
      
      if (p.column === 'left') {
        const overlap = leftCount === 0 ? 0 : 5; // Reverted to 5px
        x = (leftCount % 2 === 0) ? (centerX + allowedStagger) : (centerX - allowedStagger);
        
        y = leftUsedHeight - overlap; 
        leftUsedHeight = y + diameterPx; 
        leftCount++;
      } else {
        const overlap = rightCount === 0 ? 0 : 5; // Reverted to 5px
        x = (rightCount % 2 === 0) ? (centerX + allowedStagger) : (centerX - allowedStagger);
        
        y = rightUsedHeight - overlap;
        rightUsedHeight = y + diameterPx;
        rightCount++;
      }
      
      result.push({
        id: `prop-${index}-${p.name}`,
        colorType: colorKey as any,
        number: p.destination_id ?? '★',
        x,
        y,
        rotation,
        column: p.column,
        size: diameterPx
      });
    });
  }
  
  return result;
});

const leftStamps = computed(() => processedStamps.value.filter(s => s.column === 'left'));
const rightStamps = computed(() => processedStamps.value.filter(s => s.column === 'right'));

// ============ Expose ============
defineExpose({ clearStamps, stamps: processedStamps });
</script>

<template>
  <div class="passport-wrapper">
    <!-- Player Label -->
    <div v-if="playerName" class="player-label">
      <div v-if="character" class="player-face">
        <GameToken :type="character" />
      </div>
      <span class="player-name">{{ playerName }}</span>
    </div>

    <div class="passport-body">
      <!-- Left Column -->
      <div 
        class="column column-left"
        ref="leftColumnRef"
        @dragover="onDragOver"
        @drop="onDrop($event, 'left')"
      >
        <div 
          v-for="stamp in leftStamps" 
          :key="stamp.id"
          class="placed-stamp"
          :style="{ 
            left: `${stamp.x}px`, 
            bottom: `${stamp.y}px`,
            width: `${stamp.size}px`,
            height: `${stamp.size}px`
          }"
        >
          <Stamp 
            :colorType="stamp.colorType"
            :number="stamp.number"
            :rotation="stamp.rotation"
          />
        </div>
      </div>
      
      <!-- Right Column (Integrated hatched zone to allow overflow) -->
      <div 
        class="column column-right"
        ref="rightColumnRef"
        @dragover="onDragOver"
        @drop="onDrop($event, 'right')"
      >
        <!-- Hatched Zone (Inside the column now) -->
        <div class="hatched-zone-bg">
          <div class="m-circle">M</div>
        </div>

        <div 
          v-for="stamp in rightStamps" 
          :key="stamp.id"
          class="placed-stamp"
          :style="{ 
            left: `${stamp.x}px`, 
            bottom: `${stamp.y}px`,
            width: `${stamp.size}px`,
            height: `${stamp.size}px`
          }"
        >
          <Stamp 
            :colorType="stamp.colorType"
            :number="stamp.number"
            :rotation="stamp.rotation"
          />
        </div>
      </div>
    </div>
    
    <!-- Reset Action -->
    <button v-if="localStamps.length > 0" class="reset-btn" @click="clearStamps">
      Reset
    </button>
  </div>
</template>

<style scoped>
.passport-wrapper {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  user-select: none;
  flex-shrink: 0;
}

.player-label {
  display: flex;
  align-items: center;
  gap: 6px;
  background: rgba(255, 255, 255, 0.9);
  padding: 4px 12px;
  border-radius: 20px;
  box-shadow: 0 4px 10px rgba(0,0,0,0.1);
  border: 1px solid rgba(0,0,0,0.05);
  transform: translateY(-5px);
  z-index: 20;
  flex-shrink: 0;
}

.player-face {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
}

:deep(.player-face .token-pedestal), 
:deep(.player-face .token-shadow) {
  display: none !important;
}

.player-name {
  font-weight: 700;
  font-size: 0.9rem;
  color: #333;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.passport-body {
  width: 250px;
  height: 320px;
  background-color: #f5dcd7;
  border-radius: 12px;
  display: flex;
  justify-content: center;
  padding: 0;
  gap: 12px;
  box-shadow: 
    0 10px 30px rgba(0, 0, 0, 0.2),
    0 2px 5px rgba(0, 0, 0, 0.1);
  flex-shrink: 0;
}

.column {
  width: 100px;
  flex: none;
  background-color: #ede2da;
  position: relative;
  border-radius: 4px;
}

.column-left {
  height: 100%;
}

.column-right {
  height: 100%;
  display: flex;
  flex-direction: column;
}

/* Hatched Zone BG - Now part of the column container */
.hatched-zone-bg {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 40px;
  background: repeating-linear-gradient(
    45deg,
    rgba(139, 90, 43, 0.1),
    rgba(139, 90, 43, 0.1) 4px,
    rgba(139, 90, 43, 0.2) 4px,
    rgba(139, 90, 43, 0.2) 8px
  );
  display: flex;
  align-items: center;
  justify-content: center;
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
  z-index: 1; /* Keep it below stamps */
}

.m-circle {
  width: 28px;
  height: 28px;
  border: 1.5px solid rgba(0, 0, 0, 0.3);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: 'Times New Roman', serif;
  font-weight: bold;
  font-size: 16px;
  color: rgba(0, 0, 0, 0.4);
}

.placed-stamp {
  position: absolute;
  pointer-events: none;
  z-index: 10; /* Stamps go over the hatched zone */
}

.reset-btn {
  background: transparent;
  border: 1px solid #ccc;
  padding: 5px 15px;
  border-radius: 15px;
  color: #888;
  cursor: pointer;
  font-size: 12px;
}

.reset-btn:hover {
  background: #f0f0f0;
  color: #666;
}
</style>
