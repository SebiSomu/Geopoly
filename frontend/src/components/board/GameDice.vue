<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  value1?: number  // 1-6
  value2?: number  // 1-6
  isRolling?: boolean
  forcedDeal?: boolean
  showSecond?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  value1: 1,
  value2: 3,
  isRolling: false,
  forcedDeal: false,
  showSecond: true
})

// Standard dice: 1 opposite 6, 2 opposite 5, 3 opposite 4
// Map dice values to 3D rotations to show correct face
// Face layout:
//   - front (rotateY 0): value 1
//   - back (rotateY 180): value 6
//   - right (rotateY -90): value 3
//   - left (rotateY 90): value 4
//   - top (rotateX -90): value 2
//   - bottom (rotateX 90): value 5
const diceRotations: Record<number, string> = {
  1: 'rotateX(0deg) rotateY(0deg)',       // front - 1 pip
  2: 'rotateX(-90deg) rotateY(0deg)',     // top - 2 pips
  3: 'rotateX(0deg) rotateY(-90deg)',     // right - 3 pips
  4: 'rotateX(0deg) rotateY(90deg)',      // left - 4 pips
  5: 'rotateX(90deg) rotateY(0deg)',      // bottom - 5 pips
  6: 'rotateX(0deg) rotateY(180deg)',     // back - 6 pips
}

const dice1Transform = computed(() => {
  if (props.isRolling) {
    return 'rotateX(720deg) rotateY(720deg)'
  }
  return diceRotations[props.value1] || diceRotations[1]
})

const dice2Transform = computed(() => {
  if (props.isRolling) {
    return 'rotateX(900deg) rotateY(-900deg)'
  }
  return diceRotations[props.value2] || diceRotations[3]
})
</script>

<template>
  <div class="dice-wrapper" :class="{ rolling: isRolling }">
    <!-- Dice 1 -->
    <div class="dice-scene">
      <div class="dice dice-1" :style="{ transform: dice1Transform }">
        <!-- Front face: 1 pip (or Handshake for Forced Deal) -->
        <div class="face front">
          <div v-if="forcedDeal" class="handshake-icon">
            <svg viewBox="0 0 24 24" fill="url(#goldGrad)">
              <defs>
                <linearGradient id="goldGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                  <stop offset="0%" style="stop-color:#BF953F" />
                  <stop offset="25%" style="stop-color:#FCF6BA" />
                  <stop offset="50%" style="stop-color:#B38728" />
                  <stop offset="75%" style="stop-color:#FBF5B7" />
                  <stop offset="100%" style="stop-color:#AA771C" />
                </linearGradient>
              </defs>
              <path d="M19.3,10.1l-2.1-2.1c-0.8-0.8-2.1-0.8-2.8,0l-1,1c-0.3-0.2-0.7-0.3-1.1-0.2c-0.6,0.1-1,0.5-1.2,1L8.2,12.3 c-0.8,0.8-0.8,2.1,0,2.8l1.1,1.1c0.4,0.4,0.9,0.6,1.4,0.6c0.5,0,1-0.2,1.4-0.6l1-1h1.7c0.6,0,1.1-0.2,1.5-0.6l3-3 C20.1,12.2,20.1,10.9,19.3,10.1z" />
              <path d="M12 4.5v-1.5" stroke="url(#goldGrad)" stroke-width="1.2" stroke-linecap="round" />
              <path d="M9.5 5.5l-.8-1" stroke="url(#goldGrad)" stroke-width="1.2" stroke-linecap="round" />
              <path d="M14.5 5.5l.8-1" stroke="url(#goldGrad)" stroke-width="1.2" stroke-linecap="round" />
            </svg>
          </div>
          <div v-else class="pip center"></div>
        </div>
        <!-- Back face: 6 pips -->
        <div class="face back">
          <div class="pip top-left"></div>
          <div class="pip top-right"></div>
          <div class="pip middle-left"></div>
          <div class="pip middle-right"></div>
          <div class="pip bottom-left"></div>
          <div class="pip bottom-right"></div>
        </div>
        <!-- Right face: 3 pips -->
        <div class="face right">
          <div class="pip top-left"></div>
          <div class="pip center"></div>
          <div class="pip bottom-right"></div>
        </div>
        <!-- Left face: 4 pips -->
        <div class="face left">
          <div class="pip top-left"></div>
          <div class="pip top-right"></div>
          <div class="pip bottom-left"></div>
          <div class="pip bottom-right"></div>
        </div>
        <!-- Top face: 2 pips -->
        <div class="face top">
          <div class="pip top-left"></div>
          <div class="pip bottom-right"></div>
        </div>
        <!-- Bottom face: 5 pips -->
        <div class="face bottom">
          <div class="pip top-left"></div>
          <div class="pip top-right"></div>
          <div class="pip center"></div>
          <div class="pip bottom-left"></div>
          <div class="pip bottom-right"></div>
        </div>
      </div>
    </div>

    <!-- Dice 2 (identical face layout) -->
    <div class="dice-scene" v-if="showSecond">
      <div class="dice dice-2" :style="{ transform: dice2Transform }">
        <!-- Front face: 1 pip -->
        <div class="face front">
          <div class="pip center"></div>
        </div>
        <!-- Back face: 6 pips -->
        <div class="face back">
          <div class="pip top-left"></div>
          <div class="pip top-right"></div>
          <div class="pip middle-left"></div>
          <div class="pip middle-right"></div>
          <div class="pip bottom-left"></div>
          <div class="pip bottom-right"></div>
        </div>
        <!-- Right face: 3 pips -->
        <div class="face right">
          <div class="pip top-left"></div>
          <div class="pip center"></div>
          <div class="pip bottom-right"></div>
        </div>
        <!-- Left face: 4 pips -->
        <div class="face left">
          <div class="pip top-left"></div>
          <div class="pip top-right"></div>
          <div class="pip bottom-left"></div>
          <div class="pip bottom-right"></div>
        </div>
        <!-- Top face: 2 pips -->
        <div class="face top">
          <div class="pip top-left"></div>
          <div class="pip bottom-right"></div>
        </div>
        <!-- Bottom face: 5 pips -->
        <div class="face bottom">
          <div class="pip top-left"></div>
          <div class="pip top-right"></div>
          <div class="pip center"></div>
          <div class="pip bottom-left"></div>
          <div class="pip bottom-right"></div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dice-wrapper {
  display: flex;
  gap: 10px;
  padding: 6px;
  filter: drop-shadow(0 8px 20px rgba(0,0,0,0.5));
}

.dice-wrapper.rolling {
  animation: shake 0.3s ease-in-out;
}

@keyframes shake {
  0%, 100% { transform: translateY(0) rotate(0deg); }
  10% { transform: translateY(-6px) rotate(-2deg); }
  30% { transform: translateY(-8px) rotate(2deg); }
  50% { transform: translateY(-4px) rotate(-1deg); }
  70% { transform: translateY(-6px) rotate(1deg); }
  90% { transform: translateY(-2px) rotate(0deg); }
}

.dice-scene {
  width: 30px;
  height: 30px;
  perspective: 400px;
}

.dice {
  width: 100%;
  height: 100%;
  position: relative;
  transform-style: preserve-3d;
  transition: transform 0.5s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.dice-wrapper.rolling .dice {
  transition: transform 0.4s cubic-bezier(0.68, -0.55, 0.265, 1.55);
}

.face {
  position: absolute;
  width: 30px;
  height: 30px;
  background: radial-gradient(circle at 30% 30%, #2a2a2a, #000);
  border: 1px solid #111;
  border-radius: 4px;
  display: flex;
  justify-content: center;
  align-items: center;
  box-sizing: border-box;
  padding: 3px;
  box-shadow: 
    inset 0 0 3px rgba(255,255,255,0.05),
    inset 0 0 8px rgba(0,0,0,0.9);
}

/* Gold Engraved Pips */
.pip {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  position: absolute;
  background: linear-gradient(135deg, 
    #BF953F 0%, 
    #FCF6BA 30%, 
    #B38728 50%, 
    #FBF5B7 70%, 
    #AA771C 100%
  );
  box-shadow: 
    inset 1px 1px 1px rgba(0,0,0,0.4),
    0 1px 0 rgba(255,255,255,0.15);
}

.handshake-icon {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.handshake-icon svg {
  width: 100%;
  height: 100%;
  filter: drop-shadow(0 1px 1px rgba(0,0,0,0.3));
}

/* Pip Layouts */
.pip.center { top: 50%; left: 50%; transform: translate(-50%, -50%); }
.pip.top-left { top: 18%; left: 18%; }
.pip.top-right { top: 18%; right: 18%; }
.pip.bottom-left { bottom: 18%; left: 18%; }
.pip.bottom-right { bottom: 18%; right: 18%; }
.pip.middle-left { top: 50%; left: 18%; transform: translateY(-50%); }
.pip.middle-right { top: 50%; right: 18%; transform: translateY(-50%); }

/* 3D Face Positions */
.front  { transform: rotateY(0deg) translateZ(15px); }
.back   { transform: rotateY(180deg) translateZ(15px); }
.right  { transform: rotateY(90deg) translateZ(15px); }
.left   { transform: rotateY(-90deg) translateZ(15px); }
.top    { transform: rotateX(90deg) translateZ(15px); }
.bottom { transform: rotateX(-90deg) translateZ(15px); }

/* Animation entry */
.dice-wrapper {
  animation: dicePop 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275) forwards;
}

@keyframes dicePop {
  from { opacity: 0; transform: scale(0.5) rotate(-10deg); }
  to { opacity: 1; transform: scale(1) rotate(0deg); }
}
</style>
