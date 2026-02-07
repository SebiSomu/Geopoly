<script setup lang="ts">
import { ref } from 'vue';

const emit = defineEmits(['login-success', 'switch-to-register', 'notify']);

const username = ref('');
const password = ref('');

const handleLogin = () => {
  if (username.value && password.value) {
    // Simulate API call
    setTimeout(() => {
      emit('login-success');
    }, 500);
  } else {
    emit('notify', { message: 'Please enter both username and password', type: 'error' });
  }
};
</script>

<template>
  <div class="form-container">
    <div class="card-header red">LOGIN</div>
    <form @submit.prevent="handleLogin">
      <div class="input-group">
        <input type="text" v-model="username" placeholder="USERNAME" required />
      </div>
      <div class="input-group">
        <input type="password" v-model="password" placeholder="PASSWORD" required />
      </div>
      <button type="submit" class="submit-btn red">JOIN GAME</button>
    </form>
    <div class="switch-link">
      <span>NEW PLAYER?</span> <a href="#" @click.prevent="emit('switch-to-register')">CREATE ACCOUNT</a>
    </div>
  </div>
</template>

<style scoped>
.form-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%;
  max-width: 240px;
  background: rgba(26, 36, 51, 0.95);
  backdrop-filter: blur(8px);
  padding: 0 0 15px 0;
  border: 1px solid #c0c0c0;
  box-shadow: 0 10px 30px rgba(0,0,0,0.6);
  border-radius: 4px;
}

.card-header {
    width: 100%;
    padding: 10px 0;
    text-align: center;
    color: #00f2ff;
    font-weight: 900;
    font-size: 0.95rem;
    letter-spacing: 3px;
    margin-bottom: 1.2rem;
    border-bottom: 1px solid rgba(192, 192, 192, 0.2);
    background: rgba(0, 242, 255, 0.05);
    text-shadow: 0 0 10px rgba(0, 242, 255, 0.5);
}

form { width: 85%; }

.input-group {
  margin-bottom: 1rem;
  width: 100%;
}

input {
  width: 100%;
  padding: 10px 12px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(192, 192, 192, 0.3);
  color: white;
  font-size: 0.75rem;
  font-weight: 500;
  transition: all 0.3s ease;
  box-sizing: border-box;
  border-radius: 2px;
}

input:focus {
  outline: none;
  background: rgba(255, 255, 255, 0.08);
  border-color: #00f2ff;
  box-shadow: 0 0 10px rgba(0, 242, 255, 0.2);
}

input::placeholder {
  color: rgba(192, 192, 192, 0.4);
}

.submit-btn {
  width: 100%;
  padding: 12px;
  color: black;
  border: none;
  cursor: pointer;
  font-weight: 900;
  font-size: 0.9rem;
  margin-top: 0.3rem;
  transition: all 0.3s ease;
  text-transform: uppercase;
  letter-spacing: 2px;
  border-radius: 2px;
  background: linear-gradient(135deg, #00f2ff, #0072bb);
}

.submit-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 5px 15px rgba(0, 242, 255, 0.4);
  filter: brightness(1.1);
}

.submit-btn:active {
  transform: translateY(0);
}

.switch-link {
  margin-top: 1.2rem;
  color: rgba(192, 192, 192, 0.6);
  font-size: 0.7rem;
  font-weight: 700;
  letter-spacing: 0.8px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  text-align: center;
}

.switch-link span {
    opacity: 0.6;
    font-size: 0.6rem;
}

.switch-link a {
  color: #00f2ff;
  text-decoration: none;
  border-bottom: 1px solid rgba(0, 242, 255, 0.3);
  transition: all 0.2s;
  padding-bottom: 1px;
}

.switch-link a:hover {
    color: white;
    border-bottom-color: white;
}
</style>






