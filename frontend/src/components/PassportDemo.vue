<script setup lang="ts">
/**
 * PassportDemo.vue - Demo view for testing Passport and Stamps
 * Uses Composition API with TypeScript
 */
import { ref } from 'vue';
import Passport from './Passport.vue';
import Stamp from './Stamp.vue';
import type { StampColor } from './Stamp.vue';

// ============ Type Definitions ============
interface StampInventoryItem {
  id: string;
  colorType: StampColor;
  number: number;
  label: string;
}

// ============ Stamp Inventory (9 stamps based on Monopoly property logic) ============
const stampInventory = ref<StampInventoryItem[]>([
  { id: 'stamp-grey', colorType: 'grey', number: 0, label: 'CLASS I' },
  { id: 'stamp-brown', colorType: 'brown', number: 8, label: 'BALTIC' },
  { id: 'stamp-lightblue', colorType: 'lightblue', number: 19, label: 'ORIENTAL' },
  { id: 'stamp-pink', colorType: 'pink', number: 17, label: 'KENTUCKY' },
  { id: 'stamp-orange', colorType: 'orange', number: 13, label: 'VERMONT' },
  { id: 'stamp-red', colorType: 'red', number: 14, label: 'ILLINOIS' },
  { id: 'stamp-yellow', colorType: 'yellow', number: 15, label: 'MARVIN' },
  { id: 'stamp-green', colorType: 'green', number: 5, label: 'PACIFIC' },
  { id: 'stamp-blue', colorType: 'blue', number: 18, label: 'BOARDWALK' },
]);

// ============ Drag Handlers ============
const onDragStart = (event: DragEvent, stamp: StampInventoryItem): void => {
  if (!event.dataTransfer) return;
  
  event.dataTransfer.setData('application/json', JSON.stringify(stamp));
  event.dataTransfer.effectAllowed = 'copy';
  
  // Add visual feedback
  const target = event.target as HTMLElement;
  target.style.opacity = '0.5';
};

const onDragEnd = (event: DragEvent): void => {
  const target = event.target as HTMLElement;
  target.style.opacity = '1';
};
</script>

<template>
  <div class="demo-container">
    <!-- Sidebar: Stamp Inventory -->
    <aside class="sidebar">
      <h2 class="sidebar-title">🎫 Stamps</h2>
      <p class="sidebar-subtitle">Drag stamps to the passport!</p>
      
      <div class="stamp-grid">
        <div 
          v-for="stamp in stampInventory" 
          :key="stamp.id"
          class="stamp-wrapper"
          draggable="true"
          @dragstart="onDragStart($event, stamp)"
          @dragend="onDragEnd"
        >
          <Stamp 
            :colorType="stamp.colorType"
            :number="stamp.number"
            :label="stamp.label"
          />
        </div>
      </div>
      
      <div class="legend">
        <h3>Size Legend:</h3>
        <ul>
          <li>⭐ Grey: First Class (1.4cm)</li>
          <li>🟤 Brown → 🔵 Blue</li>
          <li>Sizes: 1.5cm → 2.5cm</li>
        </ul>
      </div>
    </aside>
    
    <!-- Main Area: Passport -->
    <main class="main-area">
      <Passport />
    </main>
  </div>
</template>

<style scoped>
.demo-container {
  display: flex;
  min-height: 100vh;
  width: 100vw;
  background: linear-gradient(135deg, #2c3e50 0%, #1a252f 100%);
  font-family: 'Segoe UI', sans-serif;
}

.sidebar {
  width: 280px;
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(10px);
  padding: 24px;
  display: flex;
  flex-direction: column;
  border-right: 1px solid rgba(255, 255, 255, 0.1);
}

.sidebar-title {
  color: white;
  font-size: 24px;
  margin: 0 0 8px 0;
}

.sidebar-subtitle {
  color: rgba(255, 255, 255, 0.6);
  font-size: 14px;
  margin: 0 0 24px 0;
}

.stamp-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  justify-content: center;
}

.stamp-wrapper {
  cursor: grab;
  transition: transform 0.2s ease;
}

.stamp-wrapper:hover {
  transform: translateY(-4px);
}

.stamp-wrapper:active {
  cursor: grabbing;
}

.legend {
  margin-top: auto;
  padding: 16px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 8px;
  color: rgba(255, 255, 255, 0.7);
  font-size: 12px;
}

.legend h3 {
  margin: 0 0 8px 0;
  font-size: 14px;
  color: white;
}

.legend ul {
  margin: 0;
  padding-left: 16px;
}

.legend li {
  margin-bottom: 4px;
}

.main-area {
  flex: 1;
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 40px;
  /* Subtle desk texture */
  background-image: 
    radial-gradient(circle at 20% 80%, rgba(120, 80, 40, 0.1) 0%, transparent 50%),
    radial-gradient(circle at 80% 20%, rgba(60, 40, 20, 0.1) 0%, transparent 50%);
}
</style>
