<script setup lang="ts">
import { } from 'vue'

interface Props {
  forcedDeal?: boolean
}

defineProps<Props>()
</script>

<template>
  <div class="dice-wrapper">
    <!-- Dice 1 -->
    <div class="dice-scene">
      <div class="dice dice-1" :style="{ transform: 'rotateX(-25deg) rotateY(35deg)' }">
        <div class="face front">
          <!-- Face 1 or Handshake -->
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
              <!-- Bold Solid Handshake (No internal gap) -->
              <path d="M19.3,10.1l-2.1-2.1c-0.8-0.8-2.1-0.8-2.8,0l-1,1c-0.3-0.2-0.7-0.3-1.1-0.2c-0.6,0.1-1,0.5-1.2,1L8.2,12.3 c-0.8,0.8-0.8,2.1,0,2.8l1.1,1.1c0.4,0.4,0.9,0.6,1.4,0.6c0.5,0,1-0.2,1.4-0.6l1-1h1.7c0.6,0,1.1-0.2,1.5-0.6l3-3 C20.1,12.2,20.1,10.9,19.3,10.1z" />
              <!-- Action lines above -->
              <path d="M12 4.5v-1.5" stroke="url(#goldGrad)" stroke-width="1.2" stroke-linecap="round" />
              <path d="M9.5 5.5l-.8-1" stroke="url(#goldGrad)" stroke-width="1.2" stroke-linecap="round" />
              <path d="M14.5 5.5l.8-1" stroke="url(#goldGrad)" stroke-width="1.2" stroke-linecap="round" />
            </svg>
          </div>
          <div v-else class="pip center"></div>
        </div>
        <div class="face back">
          <div class="pip top-left"></div>
          <div class="pip top-right"></div>
          <div class="pip center"></div>
          <div class="pip bottom-left"></div>
          <div class="pip bottom-right"></div>
          <div class="pip middle-left"></div>
        </div>
        <div class="face right">
          <div class="pip top-left"></div>
          <div class="pip bottom-right"></div>
        </div>
        <div class="face left">
          <div class="pip top-left"></div>
          <div class="pip top-right"></div>
          <div class="pip center"></div>
          <div class="pip bottom-left"></div>
          <div class="pip bottom-right"></div>
        </div>
        <div class="face top">
          <div class="pip top-left"></div>
          <div class="pip top-right"></div>
          <div class="pip bottom-left"></div>
          <div class="pip bottom-right"></div>
        </div>
        <div class="face bottom">
          <div class="pip top-left"></div>
          <div class="pip top-right"></div>
          <div class="pip center"></div>
          <div class="pip bottom-left"></div>
          <div class="pip bottom-right"></div>
        </div>
      </div>
    </div>

    <!-- Dice 2 -->
    <div class="dice-scene">
      <div class="dice dice-2" :style="{ transform: 'rotateX(-20deg) rotateY(-30deg)' }">
        <div class="face front">
          <div class="pip top-left"></div>
          <div class="pip center"></div>
          <div class="pip bottom-right"></div>
        </div>
        <div class="face back">
          <div class="pip top-left"></div>
          <div class="pip top-right"></div>
          <div class="pip middle-left"></div>
          <div class="pip middle-right"></div>
          <div class="pip bottom-left"></div>
          <div class="pip bottom-right"></div>
        </div>
        <div class="face right">
          <div class="pip center"></div>
        </div>
        <div class="face left">
          <div class="pip top-left"></div>
          <div class="pip top-right"></div>
          <div class="pip bottom-left"></div>
          <div class="pip bottom-right"></div>
        </div>
        <div class="face top">
          <div class="pip top-left"></div>
          <div class="pip bottom-right"></div>
        </div>
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
  gap: 12px;
  padding: 8px;
  filter: drop-shadow(0 12px 25px rgba(0,0,0,0.6));
}

.dice-scene {
  width: 38px;
  height: 38px;
  perspective: 400px;
}

.dice {
  width: 100%;
  height: 100%;
  position: relative;
  transform-style: preserve-3d;
  transition: transform 0.8s cubic-bezier(0.175, 0.885, 0.32, 1.275);
}

.face {
  position: absolute;
  width: 38px;
  height: 38px;
  background: radial-gradient(circle at 30% 30%, #222, #000);
  border: 1px solid #111;
  border-radius: 5px;
  display: flex;
  justify-content: center;
  align-items: center;
  box-sizing: border-box;
  padding: 4px;
  box-shadow: 
    inset 0 0 4px rgba(255,255,255,0.05),
    inset 0 0 10px rgba(0,0,0,0.9);
}

/* Gold Engraved Pips */
.pip {
  width: 6px;
  height: 6px;
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
  width: 31px;
  height: 31px;
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
.front  { transform: rotateY(0deg) translateZ(19px); }
.back   { transform: rotateY(180deg) translateZ(19px); }
.right  { transform: rotateY(90deg) translateZ(19px); }
.left   { transform: rotateY(-90deg) translateZ(19px); }
.top    { transform: rotateX(90deg) translateZ(19px); }
.bottom { transform: rotateX(-90deg) translateZ(19px); }

/* Animation entry */
.dice-wrapper {
  animation: dicePop 1s cubic-bezier(0.175, 0.885, 0.32, 1.275) forwards;
}

@keyframes dicePop {
  from { opacity: 0; transform: scale(0.5) rotate(-20deg); }
  to { opacity: 1; transform: scale(1) rotate(0deg); }
}
</style>
