<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import LoginForm from './LoginForm.vue';
import RegisterForm from './RegisterForm.vue';
import Footer from './Footer.vue';
import Notification from './Notification.vue';
import RulesModal from './RulesModal.vue';

const router = useRouter();
const viewState = ref<'start' | 'login' | 'register'>('start');
const notification = ref<{ message: string; type: 'error' | 'success' | 'info' } | null>(null);
const showRules = ref(false);

const toggleRules = () => {
    showRules.value = !showRules.value;
};

const showLogin = () => {
  const savedUser = localStorage.getItem('username');
  if (savedUser) {
      // Auto-login
      router.push({ name: 'Lobby' });
      return;
  }
  viewState.value = 'login';
};

const showRegister = () => {
  viewState.value = 'register';
};

const showStart = () => {
  viewState.value = 'start';
}

const handleLoginSuccess = () => {
    notification.value = { message: 'WELCOME BACK!', type: 'success' };
    setTimeout(() => {
        router.push({ name: 'Lobby' });
    }, 1000);
};

const handleRegisterSuccess = () => {
    notification.value = { message: 'ACCOUNT CREATED!', type: 'success' };
    setTimeout(() => {
        router.push({ name: 'Lobby' });
    }, 1000);
}

const handleNotify = (payload: { message: string; type: 'error' | 'success' | 'info' }) => {
    notification.value = payload;
}

const clearNotification = () => {
    notification.value = null;
}

</script>

<template>
    <div class="start-screen">
        <div class="overlay"></div>
        
        <transition name="fade" mode="out-in">
            <div v-if="viewState === 'start'" class="center-content" key="start">
                <div class="logo-container">
                    <h1 class="title">
                        <span class="geo">GEO</span><span class="poly">POLY</span>
                    </h1>
                    <p class="subtitle"></p>
                </div>
                <button class="start-btn" @click="showLogin">START GAME</button>
                
                <!-- Rules Button -->
                <button class="rules-btn" @click="toggleRules" title="Game Rules">
                    <svg viewBox="0 0 24 24" class="book-icon">
                        <path fill="currentColor" d="M18,3H6C4.9,3,4,3.9,4,5v14c0,1.1,0.9,2,2,2h12c1.1,0,2-0.9,2-2V5C20,3.9,19.1,3,18,3z M18,19H6V5h1v14h1V5h10V19z M9,7h7v2H9V7z M9,11h7v2H9V11z M9,15h7v2H9V15z"/>
                    </svg>
                </button>
            </div>

            <div v-else-if="viewState === 'login'" class="center-content form-view" key="login">
                 <LoginForm 
                    @login-success="handleLoginSuccess" 
                    @switch-to-register="showRegister" 
                    @notify="handleNotify"
                />
                 <button class="back-btn" @click="showStart">BACK TO MAIN</button>
            </div>

            <div v-else-if="viewState === 'register'" class="center-content form-view" key="register">
                <RegisterForm 
                    @register-success="handleRegisterSuccess" 
                    @switch-to-login="showLogin" 
                    @notify="handleNotify"
                />
                 <button class="back-btn" @click="showStart">BACK TO MAIN</button>
            </div>
        </transition>

        <Notification 
            v-if="notification" 
            :message="notification.message" 
            :type="notification.type" 
            @close="clearNotification"
        />

        <RulesModal :is-open="showRules" @close="toggleRules" />

        <Footer />
    </div>
</template>

<style scoped>
.start-screen {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background-color: #000;
    background-image: 
        radial-gradient(circle at 50% 50%, #001a33 0%, #000 100%);
    color: white;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    overflow: hidden;
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
}

.overlay {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: url('https://www.transparenttextures.com/patterns/dark-matter.png');
    opacity: 0.05;
    pointer-events: none;
}

.center-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    z-index: 10;
    padding-bottom: 30px; 
}

.logo-container {
    text-align: center;
    margin-bottom: 3rem;
}

.title {
    font-family: 'Arial', 'Helvetica', sans-serif;
    font-size: 80px;
    font-weight: 700;
    letter-spacing: 8px;
    text-transform: uppercase;
    display: flex;
    justify-content: center;
    margin: 0;
}

/* NEON OUTLINE - GEO */
.geo {
    color: transparent;
    -webkit-text-stroke: 2.5px #00ffff;
    text-stroke: 2.5px #00ffff;
    text-shadow: 
        0 0 10px #00ffff,
        0 0 20px #00ffff,
        0 0 30px #00ffff;
}

/* NEON OUTLINE - POLY */
.poly {
    color: transparent;
    -webkit-text-stroke: 2.5px #ffff00;
    text-stroke: 2.5px #ffff00;
    text-shadow: 
        0 0 10px #ffff00,
        0 0 20px #ffff00,
        0 0 30px #ffff00;
}

.start-btn {
    padding: 10px 40px;
    font-size: 0.95rem;
    background: linear-gradient(135deg, #0072BB, #00467F);
    color: white;
    border: 1px solid #c0c0c0; /* Modern silver border */
    cursor: pointer;
    transition: all 0.3s ease;
    letter-spacing: 2px;
    font-weight: 900;
    text-transform: uppercase;
    box-shadow: 4px 4px 0px rgba(0,0,0,0.3);
}

.start-btn:hover {
    transform: translate(-1px, -1px);
    box-shadow: 6px 6px 0px rgba(0,0,0,0.3);
    filter: brightness(1.1);
}

.start-btn:active {
    transform: translate(0, 0);
    box-shadow: 2px 2px 0px rgba(0,0,0,0.3);
}

.back-btn {
    margin-top: 25px;
    background: transparent;
    border: none;
    color: #666;
    cursor: pointer;
    text-transform: uppercase;
    font-weight: 900;
    letter-spacing: 1px;
    font-size: 0.7rem;
    transition: color 0.3s;
}

.back-btn:hover {
    color: #0072BB;
    text-decoration: underline;
}

/* Transitions */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease, transform 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateY(10px);
}

/* Rules Button */
.rules-btn {
    margin-top: 2rem;
    width: 60px;
    height: 60px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(0, 210, 255, 0.3);
    color: #00d2ff;
    display: flex;
    justify-content: center;
    align-items: center;
    cursor: pointer;
    transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
    box-shadow: 0 0 15px rgba(0, 210, 255, 0.1);
}

.rules-btn:hover {
    background: rgba(0, 210, 255, 0.15);
    transform: scale(1.15) rotate(5deg);
    border-color: #00d2ff;
    box-shadow: 0 0 30px rgba(0, 210, 255, 0.4);
    color: #fff;
}

.book-icon {
    width: 28px;
    height: 28px;
    filter: drop-shadow(0 0 5px rgba(0, 210, 255, 0.5));
}
</style>


