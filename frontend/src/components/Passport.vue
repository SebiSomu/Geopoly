<script setup lang="ts">
/**
 * Passport.vue - Realistic single-page passport with 2 columns
 * Uses Composition API with TypeScript
 */
import { ref, computed } from 'vue';
import Stamp from './Stamp.vue';
import type { StampColor } from './Stamp.vue';

// ============ Type Definitions ============
export interface PlacedStamp {
  id: string;
  colorType: StampColor;
  number: number | string;
  x: number;
  y: number;
  rotation: number;
  column: 'left' | 'right';
}

// ============ State ============
const stamps = ref<PlacedStamp[]>([]);
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
    const y = event.clientY - columnRect.top - (droppedStamp.size || 35);
    const rotation = Math.floor(Math.random() * 16) - 8;

    stamps.value.push({
      id: `stamp-${Date.now()}-${Math.random().toString(36).substr(2, 5)}`,
      colorType: droppedStamp.colorType,
      number: droppedStamp.number,
      x,
      y,
      rotation,
      column,
    });
  } catch (e) {
    console.error('Failed to parse dropped stamp data:', e);
  }
};

// ============ Clear Stamps ============
const clearStamps = (): void => {
  stamps.value = [];
};

// ============ Computed Filters ============
const leftStamps = computed(() => stamps.value.filter(s => s.column === 'left'));
const rightStamps = computed(() => stamps.value.filter(s => s.column === 'right'));

// ============ Expose ============
defineExpose({ clearStamps, stamps });
</script>

<template>
  <div class="passport-wrapper">
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
          :style="{ left: `${stamp.x}px`, top: `${stamp.y}px` }"
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
          :style="{ left: `${stamp.x}px`, top: `${stamp.y}px` }"
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
    <button v-if="stamps.length > 0" class="reset-btn" @click="clearStamps">
      Reset
    </button>
  </div>
</template>

<style scoped>
.passport-wrapper {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 15px;
  user-select: none;
}

.passport-body {
  width: 220px;
  height: 360px;
  background-color: #f5dcd7;
  border-radius: 12px;
  display: flex;
  justify-content: center;
  padding: 12px;
  gap: 10px;
  box-shadow: 
    0 10px 30px rgba(0, 0, 0, 0.2),
    0 2px 5px rgba(0, 0, 0, 0.1);
}

.column {
  flex: 1;
  background-color: #ede2da;
  position: relative;
  overflow: hidden;
  border-radius: 4px;
  box-shadow: 
    inset 2px 2px 5px rgba(0, 0, 0, 0.2),
    inset -1px -1px 2px rgba(255, 255, 255, 0.4);
  border: 1px solid rgba(0, 0, 0, 0.1);
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
  height: 48px;
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
