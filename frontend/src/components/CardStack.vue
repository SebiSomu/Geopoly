<script setup lang="ts">
interface Props {
  type: 'chance' | 'here_and_now'
}

defineProps<Props>()
</script>

<template>
  <div class="card-stack" :class="type">
    <!-- Multiple layers for stack effect -->
    <div class="card-layer plate-3"></div>
    <div class="card-layer plate-2"></div>
    <div class="card-layer plate-1"></div>
    
    <!-- Top card -->
    <div class="card-top">
      <div class="card-content">
        <template v-if="type === 'chance'">
          <div class="chance-logo">
            <span class="question-mark">?</span>
            <span class="label">CHANCE</span>
          </div>
        </template>
        <template v-else>
          <div class="here-now-logo">
            <div class="globe">🌎</div>
            <div class="here-now-text">
              <span class="text-here">HERE</span>
              <span class="text-and">&</span>
              <span class="text-now">NOW</span>
            </div>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped>
.card-stack {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.card-layer {
  position: absolute;
  width: 100%;
  height: 100%;
  border-radius: 8px;
  background: white;
  border: 1px solid rgba(0,0,0,0.1);
  box-shadow: 2px 2px 5px rgba(0,0,0,0.2);
}

.plate-3 { transform: translate(4px, 4px); opacity: 0.6; }
.plate-2 { transform: translate(2px, 2px); opacity: 0.8; }
.plate-1 { transform: translate(0, 0); }

.card-top {
  position: absolute;
  width: 100%;
  height: 100%;
  background: white;
  border-radius: 8px;
  border: 1px solid rgba(0,0,0,0.1);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  box-shadow: 2px 2px 10px rgba(0,0,0,0.3);
}

.card-content {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* Chance Styling */
.chance .card-top {
  background: linear-gradient(135deg, #9c27b0, #7b1fa2);
}

.chance-logo {
  display: flex;
  flex-direction: column;
  align-items: center;
  color: white;
  transform: rotate(90deg);
}

.question-mark {
  font-family: 'Oswald', sans-serif;
  font-size: 1.5rem;
  font-weight: 700;
  line-height: 1;
}

.label {
  font-family: 'Oswald', sans-serif;
  font-size: 0.5rem;
  font-weight: 400;
  letter-spacing: 1px;
}

/* Here & Now Styling */
.here_and_now .card-top {
  background: linear-gradient(135deg, #cc0000, #990000);
}

.here-now-logo {
  display: flex;
  flex-direction: column;
  align-items: center;
  color: white;
  transform: rotate(-90deg);
}

.globe {
  font-size: 0.9rem;
  margin-bottom: 1px;
}

.here-now-text {
  display: flex;
  flex-direction: column;
  align-items: center;
  line-height: 1;
}

.text-here { font-family: 'Oswald', sans-serif; font-size: 0.4rem; font-weight: 700; }
.text-and { font-family: 'Oswald', sans-serif; font-size: 0.3rem; color: #ccc; }
.text-now { font-family: 'Oswald', sans-serif; font-size: 0.5rem; font-weight: 700; }
</style>
