<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';

defineProps<{
  isOpen: boolean;
}>();

const emit = defineEmits(['close']);

const close = () => {
  emit('close');
};

const handleEscape = (e: KeyboardEvent) => {
  if (e.key === 'Escape') close();
};

onMounted(() => window.addEventListener('keydown', handleEscape));
onUnmounted(() => window.removeEventListener('keydown', handleEscape));
</script>

<template>
  <Teleport to="body">
    <Transition name="premium-modal">
      <div v-if="isOpen" class="modal-overlay" @click.self="close">
        <div class="modal-container">
          <!-- Glassmorphism Header -->
          <header class="modal-header">
            <div class="title-wrapper">
              <span class="neon-line"></span>
              <h2>GEOPOLY GUIDE</h2>
              <span class="neon-line"></span>
            </div>
            <button class="close-btn" @click="close" aria-label="Close rules">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <path d="M18 6L6 18M6 6l12 12" />
              </svg>
            </button>
          </header>

          <div class="modal-content custom-scrollbar">
            <section class="info-card objective">
              <div class="icon-box">🌍</div>
              <div class="text-box">
                <h3>THE OBJECTIVE</h3>
                <p>Become the world's greatest traveler! Fill your passport with stamps from exotic destinations, build your empire, and outlast your rivals in this high-stakes global race.</p>
              </div>
            </section>

            <section class="info-card preparation">
              <div class="card-header">
                <span class="step-num">START</span>
                <h3>INITIAL SETUP</h3>
              </div>
              <p>Every player begins their journey with <strong>G1500</strong> and <strong>2 "Here & Now" cards</strong>. Use them wisely!</p>
            </section>

            <div class="grid-sections">
              <section class="info-card">
                <div class="card-header">
                  <span class="step-num">01</span>
                  <h3>MOVEMENT</h3>
                </div>
                <p>Roll the dual dice to navigate. Rolling <strong>doubles</strong> lets you move and roll again, but beware: 3 doubles in a row sends you straight to Jail!</p>
              </section>

              <section class="info-card highlight">
                <div class="card-header">
                  <span class="step-num">🤝</span>
                  <h3>FORCED DEAL</h3>
                </div>
                <p>Rolled the Business icon? You triggered a <strong>Forced Deal</strong>! You can choose to <strong>Swap</strong> your last stamp with an opponent's last stamp, or simply <strong>Move</strong> the steps on your die.</p>
              </section>

              <section class="info-card">
                <div class="card-header">
                  <span class="step-num">02</span>
                  <h3>STAMPS</h3>
                </div>
                <p>Purchase destinations to earn unique stamps. Collect full color sets to boost your destination's value and prestige with <strong>First Class</strong> upgrades.</p>
              </section>

              <section class="info-card">
                <div class="card-header">
                  <span class="step-num">03</span>
                  <h3>TAXES</h3>
                </div>
                <p>Landing on owned territory? Prepare to pay the <strong>Tourist Tax</strong>. The tax increases significantly for every stamp in the owner's passport!</p>
              </section>
            </div>

            <section class="info-card card-list highlight">
                <div class="card-header">
                  <span class="step-num">⚡</span>
                  <h3>HERE & NOW CARDS</h3>
                </div>
                <p class="usage-note"><strong>TIMING:</strong> These cards can be played <strong>at any time</strong> (even during someone else's turn), provided the action makes logical sense in the current situation.</p>
                <div class="scroll-list full-cards">
                    <div class="card-group">
                        <h4>OFFENSIVE & STRATEGIC</h4>
                        <p>• <strong>Just Say No! (x3):</strong> The ultimate counter. Use it to cancel ANY action targeted at you.</p>
                        <p>• <strong>Intercept:</strong> Buy a property an opponent just purchased. You pay the price, they get a refund.</p>
                        <p>• <strong>Steal First Class:</strong> Steal a "First Class" stamp from an opponent the moment they earn it.</p>
                        <p>• <strong>Swap Stamps (x2):</strong> Exchange your last stamp with any opponent's last stamp.</p>
                        <p>• <strong>Take All:</strong> Removes the last stamp from EVERY player and returns them to the board bank.</p>
                    </div>
                    <div class="card-group">
                        <h4>FINANCIAL & MOVEMENT</h4>
                        <p>• <strong>Tax Amnesty:</strong> Sell any stamp from your passport back to the bank for 150% of its price.</p>
                        <p>• <strong>Collect Tax:</strong> When landing on an opponent's property, you collect the tax instead of paying it.</p>
                        <p>• <strong>Collect from Richest:</strong> The player with the most stamps must pay you G200.</p>
                        <p>• <strong>Discount Purchase:</strong> Pay only G100 for any unowned property you land on.</p>
                        <p>• <strong>Move Anywhere:</strong> Teleport your token to any space on the entire board.</p>
                        <p>• <strong>Advance 5:</strong> Instantly move forward 5 spaces.</p>
                        <p>• <strong>Block Double:</strong> Pick an opponent; their next double roll will move them but end their turn immediately.</p>
                        <p>• <strong>Jail Escape:</strong> Keep this to get out of Jail for free.</p>
                    </div>
                </div>
            </section>

            <section class="info-card card-list">
                <div class="card-header">
                  <span class="step-num">❓</span>
                  <h3>CHANCE CARDS</h3>
                </div>
                <p class="usage-note">These cards are triggered <strong>instantly</strong> when you land on a Chance space. You must follow the instructions immediately.</p>
                <div class="scroll-list full-cards">
                    <div class="card-group">
                        <p>• <strong>Steal & Pay:</strong> Take an opponent's last stamp, but you must pay them its full original value.</p>
                        <p>• <strong>Dice Challenge (x2):</strong> Duel an opponent! Both roll a die; highest roll wins G100 from the other.</p>
                        <p>• <strong>Free Parking:</strong> Teleport to Free Parking and collect all recently paid tourist taxes from the pot.</p>
                        <p>• <strong>Swap Two:</strong> Pick two other players; they must swap their last stamps with each other.</p>
                        <p>• <strong>First Class Bonus (x2):</strong> Collect G40 from the bank for every "First Class" stamp in your passport.</p>
                        <p>• <strong>Reroll:</strong> Not happy? Reroll one of your dice and move the new total.</p>
                        <p>• <strong>Hospital Tax:</strong> Pay a G200 fee to the bank.</p>
                        <p>• <strong>Collect G100 (x2):</strong> Take a G100 bonus from the bank.</p>
                        <p>• <strong>Collect from All:</strong> Every other player must pay you G40.</p>
                        <p>• <strong>Advance to Start:</strong> Go directly to START and collect G200.</p>
                        <p>• <strong>Advance 5:</strong> Move forward 5 spaces and handle the landing.</p>
                        <p>• <strong>Go to Jail:</strong> Your turn ends. Go directly to Jail.</p>
                        <p>• <strong>Jail Escape:</strong> Keep this to get out of Jail for free.</p>
                    </div>
                </div>
            </section>

            <section class="info-card victory">
              <div class="victory-banner">
                <h3>PATH TO VICTORY</h3>
                <p>The game ends when a player fills their passport completely or when all other players are bankrupt. Glory awaits the boldest investor!</p>
              </div>
            </section>
          </div>

          <footer class="modal-footer">
            <button class="confirm-btn" @click="close">I'M READY</button>
          </footer>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 7, 20, 0.8);
  backdrop-filter: blur(8px);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 9999;
  padding: 20px;
  will-change: opacity;
}

.modal-container {
  background: #000c14; /* Solid background for better scroll perf */
  background-image: linear-gradient(165deg, #001220 0%, #00080f 100%);
  width: 100%;
  max-width: 680px;
  max-height: 85vh;
  border: 1px solid rgba(0, 210, 255, 0.2);
  border-radius: 20px;
  display: flex;
  flex-direction: column;
  position: relative;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.8);
  overflow: hidden;
  transform: translateZ(0); /* Hardware acceleration */
  backface-visibility: hidden;
}

.modal-header {
  padding: 20px 32px;
  display: flex;
  justify-content: center;
  align-items: center;
  position: relative;
  background: rgba(255, 255, 255, 0.03);
  border-bottom: 1px solid rgba(0, 210, 255, 0.1);
}

.title-wrapper {
  display: flex;
  align-items: center;
  gap: 15px;
}

.neon-line {
  height: 2px;
  width: 30px;
  background: linear-gradient(90deg, transparent, #00d2ff);
}

.title-wrapper .neon-line:last-child {
  background: linear-gradient(90deg, #00d2ff, transparent);
}

h2 {
  margin: 0;
  font-size: 1.3rem;
  letter-spacing: 4px;
  color: #00d2ff;
  font-weight: 900;
  text-transform: uppercase;
}

.close-btn {
  position: absolute;
  right: 20px;
  background: rgba(255, 255, 255, 0.05);
  border: none;
  color: #fff;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  padding: 6px;
  cursor: pointer;
  transition: transform 0.2s ease, background 0.2s ease;
}

.close-btn:hover {
  background: rgba(255, 50, 50, 0.2);
  color: #ff4d4d;
  transform: rotate(90deg);
}

.modal-content {
  padding: 24px 32px;
  overflow-y: auto;
  flex: 1;
  /* Ultimate scroll performance: No smooth scroll, native feel */
  -webkit-overflow-scrolling: touch;
}

.info-card {
  background: #001a2c; /* Solid color instead of semi-transparent */
  border: 1px solid #002d4d;
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 12px;
  /* Remove hover transitions/transforms for better scroll performance */
}

.objective {
  display: flex;
  gap: 15px;
  align-items: center;
  background: #00243d;
}

.icon-box {
  font-size: 2rem;
}

.grid-sections {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
  margin-bottom: 12px;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.step-num {
  font-size: 0.75rem;
  font-weight: 900;
  color: #ffd700;
  background: rgba(255, 215, 0, 0.1);
  padding: 1px 6px;
  border-radius: 3px;
}

.usage-note {
  font-size: 0.85rem;
  color: #00d2ff;
  background: rgba(0, 210, 255, 0.05);
  padding: 8px 12px;
  border-radius: 6px;
  margin: 10px 0 15px 0;
  border-left: 2px solid #00d2ff;
}

.card-group {
  margin-bottom: 20px;
}

.card-group h4 {
  color: #ffd700;
  font-size: 0.8rem;
  letter-spacing: 2px;
  margin-bottom: 10px;
  opacity: 0.8;
}

.full-cards p {
  margin-bottom: 8px;
  font-size: 0.85rem;
}

.full-cards strong {
  color: #fff;
}

h3 {
  margin: 0;
  font-size: 0.9rem;
  letter-spacing: 1px;
  color: #fff;
  text-transform: uppercase;
}

p {
  margin: 0;
  color: #888;
  font-size: 0.9rem;
  line-height: 1.4;
}

.victory {
  text-align: center;
  background: rgba(255, 215, 0, 0.02);
  margin-bottom: 0;
}

.victory h3 {
  color: #ffd700;
}

.modal-footer {
  padding: 20px 32px;
  display: flex;
  justify-content: center;
  background: rgba(0, 0, 0, 0.3);
  border-top: 1px solid rgba(255, 255, 255, 0.03);
}

.confirm-btn {
  background: #00d2ff;
  background: linear-gradient(90deg, #00d2ff, #0072BB);
  border: none;
  padding: 10px 40px;
  border-radius: 10px;
  color: white;
  font-weight: 900;
  letter-spacing: 2px;
  cursor: pointer;
  transition: transform 0.2s ease, filter 0.2s ease;
}

.confirm-btn:hover {
  transform: translateY(-2px);
  filter: brightness(1.1);
}

/* Scrollbar */
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(0, 210, 255, 0.3);
  border-radius: 10px;
}

/* Transitions */
.premium-modal-enter-active,
.premium-modal-leave-active {
  transition: opacity 0.25s ease, transform 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  will-change: transform, opacity;
}

.premium-modal-enter-from {
  opacity: 0;
  transform: scale(0.96);
}

.premium-modal-leave-to {
  opacity: 0;
  transform: scale(0.98);
}

@media (max-width: 600px) {
  .grid-sections {
    grid-template-columns: 1fr;
  }
}
</style>
