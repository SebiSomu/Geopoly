<script setup lang="ts">
/**
 * Stamp.vue - High-fidelity circular stamp for Monopoly World Edition
 * Features: Circular text, side dots, matte ink look.
 */
import { computed, type CSSProperties } from 'vue';

// ============ Type Definitions ============
export type StampColor = 
  | 'grey' | 'brown' | 'lightblue' | 'pink' | 'orange' 
  | 'red'  | 'yellow'| 'green' | 'blue';

export interface StampProps {
  colorType: StampColor;
  number?: number | string;
  label?: string;    // Property name (e.g., "BALTIC")
  rotation?: number;
}

// ============ Dimension Mapping (cm to px) ============
const STAMP_SIZES: Record<StampColor, number> = {
  grey: 53, brown: 57, lightblue: 60, pink: 68, orange: 70, 
  red: 79, yellow: 83, green: 93, blue: 95,
};

// ============ Matte Colors (Reference matched) ============
const COLORS: Record<StampColor, string> = {
  grey: '#a6a6a6',
  brown: '#8B4513',
  lightblue: '#79F7EF',
  pink: '#D61A8B',
  orange: '#F0760C',
  red: '#CC0000',
  yellow: '#FBFF00',
  green: '#04910D',
  blue: '#0D47A1',
};

const props = withDefaults(defineProps<StampProps>(), {
  colorType: 'red',
  number: '',
  label: 'LONDON',
  rotation: 0,
});

const size = computed(() => STAMP_SIZES[props.colorType]);
const color = computed(() => COLORS[props.colorType]);
const isFirstClass = computed(() => props.colorType === 'grey');

// Helper for unique filter/path IDs to avoid collisions
const id = Math.random().toString(36).substring(2, 7);

const stampStyle = computed<CSSProperties>(() => ({
  width: `${size.value}px`,
  height: `${size.value}px`,
  backgroundColor: color.value,
  transform: `rotate(${props.rotation}deg)`,
} as CSSProperties));
</script>

<template>
  <div class="stamp" :style="stampStyle">
    <svg 
      class="stamp-svg" 
      :viewBox="`0 0 100 100`" 
      xmlns="http://www.w3.org/2000/svg"
    >
      <defs>
        <!-- Circular path for Top text (MONOPOLY) -->
        <path :id="`topPath-${id}`" d="M 20,50 A 30,30 0 1,1 80,50" />
        <!-- Circular path for Bottom text (PROPERTY NAME) -->
        <path :id="`bottomPath-${id}`" d="M 20,50 A 30,30 0 0,0 80,50" />
        
        <!-- Faded ink texture -->
        <filter :id="`inkFilter-${id}`">
          <feTurbulence type="fractalNoise" baseFrequency="0.6" numOctaves="3" result="noise" />
          <feDisplacementMap in="SourceGraphic" in2="noise" scale="1" />
        </filter>
      </defs>

      <!-- Outer thin ring -->
      <circle cx="50" cy="50" r="48" fill="none" stroke="rgba(0,0,0,0.15)" stroke-width="0.5" />
      
      <!-- Middle ring (thick ink area) -->
      <circle cx="50" cy="50" r="38" fill="none" stroke="black" stroke-width="0.8" opacity="0.3" />
      
      <!-- Inner core circle -->
      <circle cx="50" cy="50" r="28" fill="none" stroke="black" stroke-width="0.5" opacity="0.2" />

      <!-- Side dots -->
      <circle cx="18" cy="50" r="1.5" fill="black" opacity="0.4" />
      <circle cx="82" cy="50" r="1.5" fill="black" opacity="0.4" />

      <!-- Center Image/Number -->
      <g :filter="`url(#inkFilter-${id})`">
        <template v-if="isFirstClass">
          <!-- Realistic Star for Class I -->
          <polygon 
            points="50,32 55,44 68,44 58,52 62,65 50,57 38,65 42,52 32,44 45,44" 
            fill="rgba(0,0,0,0.6)" 
          />
          <polygon 
            points="50,38 53,46 62,46 55,51 58,60 50,54 42,60 45,51 38,46 47,46" 
            fill="none" 
            stroke="rgba(0,0,0,0.4)" 
            stroke-width="0.5"
          />
        </template>
        <text 
          v-else 
          x="50" 
          y="58" 
          font-size="25"
          text-anchor="middle" 
          font-weight="900" 
          fill="rgba(0,0,0,0.7)"
          font-family="'Arial Black', sans-serif"
        >
          {{ number }}
        </text>
      </g>
    </svg>
  </div>
</template>

<style scoped>
.stamp {
  border-radius: 50%;
  border: 1.5px solid rgba(0, 0, 0, 0.2);
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  box-shadow: 
    0 3px 6px rgba(0, 0, 0, 0.2),
    inset 0 1px 2px rgba(255, 255, 255, 0.2);
  overflow: hidden;
  transition: transform 0.1s ease;
}

.stamp:active {
  transform: scale(0.96);
}

.stamp-svg {
  width: 100%;
  height: 100%;
}

text {
  font-family: 'Verdana', sans-serif;
  fill: black;
}
</style>
