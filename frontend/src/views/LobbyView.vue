<script setup lang="ts">
import { ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useMutation, useQuery } from '@vue/apollo-composable';
import { 
    CREATE_LOBBY_MUTATION, 
    JOIN_LOBBY_MUTATION, 
    SELECT_CHARACTER_MUTATION, 
    GET_LOBBY_QUERY 
} from '../graphql/operations';
import Footer from '../components/login/Footer.vue';
import CharacterSelection from '../components/lobby/CharacterSelection.vue';

const router = useRouter();
const username = ref(localStorage.getItem('username') || 'Guest');
const lobbyCode = ref('');
const inputCode = ref('');
const error = ref('');
const isInLobby = ref(false);

// Characters matching GameToken types
const characters = [
    { id: 'dog', name: 'Dog' },
    { id: 'cat',  name: 'Cat' },
    { id: 'seal', name: 'Seal' },
    { id: 'capybara', name: 'Capybara' },
];

const { mutate: createLobby, onDone: onCreateDone, onError: onCreateError } = useMutation(CREATE_LOBBY_MUTATION);
const { mutate: joinLobby, onDone: onJoinDone, onError: onJoinError } = useMutation(JOIN_LOBBY_MUTATION);
const { mutate: selectCharacter } = useMutation(SELECT_CHARACTER_MUTATION);

// Polling for lobby state
const { result } = useQuery(GET_LOBBY_QUERY, () => ({
    code: lobbyCode.value
}), {
    enabled: computed(() => !!lobbyCode.value),
    pollInterval: 1000 
});

const lobbyState = computed(() => result.value?.getLobby);
const players = computed(() => lobbyState.value?.players || []);

const handleCreate = () => {
    error.value = '';
    // Explicitly casting variables if needed, though usually automatic
    if (createLobby) {
        createLobby({ username: username.value });
    }
};

onCreateDone((res) => {
    if (res.data?.createLobby) {
        lobbyCode.value = res.data.createLobby.code;
        isInLobby.value = true;
    }
});

onCreateError((err) => {
    error.value = err.message;
});

const handleJoin = () => {
    error.value = '';
    if (inputCode.value.length !== 6) {
        error.value = "Code must be 6 characters";
        return;
    }
    if (joinLobby) {
        joinLobby({ code: inputCode.value.toUpperCase(), username: username.value });
    }
};

onJoinDone((res) => {
    if (res.data?.joinLobby) {
        lobbyCode.value = res.data.joinLobby.code;
        isInLobby.value = true;
    }
});

onJoinError((err) => {
    error.value = err.message;
});

const handleSelectChar = (charId: string) => {
    if (players.value.some((p: any) => p.character === charId)) return;
    
    if (selectCharacter) {
        selectCharacter({ 
            code: lobbyCode.value, 
            username: username.value, 
            character: charId 
        }).catch(err => error.value = err.message);
    }
};

const startGame = () => {
    router.push({ name: 'Game', params: { code: lobbyCode.value } });
};

</script>

<template>
    <div class="lobby-screen">
        <div class="overlay"></div>
        
        <div class="content-container">
            <h1 class="lobby-title">LOBBY</h1>
            
            <div v-if="error" class="error-msg">{{ error }}</div>

            <div v-if="!isInLobby" class="actions">
                <div class="action-box">
                    <h2>CREATE NEW GAME</h2>
                    <p>Host a game and invite your friends.</p>
                    <button class="action-btn" @click="handleCreate">CREATE LOBBY</button>
                </div>
                
                <div class="divider">OR</div>

                <div class="action-box">
                    <h2>JOIN GAME</h2>
                    <p>Enter the 6-character code.</p>
                    <input v-model="inputCode" placeholder="ABC123" maxlength="6" class="code-input" />
                    <button class="action-btn" @click="handleJoin">JOIN LOBBY</button>
                </div>
            </div>

            <div v-else class="lobby-room">
                <div class="room-header">
                    <span>ROOM CODE:</span>
                    <span class="code-display">{{ lobbyCode }}</span>
                </div>

                <div class="players-list">
                    <h3>PLAYERS ({{ players.length }}/4)</h3>
                    <ul>
                        <li v-for="player in players" :key="player.username" :class="{ 'self': player.username === username }">
                            <span class="p-name">{{ player.username }}</span>
                            <span class="p-status" v-if="player.character">
                                <!-- Small preview icon could go here if needed -->
                                Ready
                            </span>
                            <span class="p-status waiting" v-else>Thinking...</span>
                        </li>
                    </ul>
                </div>

                <div class="char-selection">
                    <h3>CHOOSE YOUR PIECE</h3>
                    <div class="char-bar">
                        <CharacterSelection 
                            v-for="char in characters"
                            :key="char.id"
                            :type="char.id"
                            :selected="players.find((p: any) => p.username === username)?.character === char.id"
                            :taken="players.some((p: any) => p.character === char.id && p.username !== username)"
                            @select="handleSelectChar(char.id)"
                        />
                    </div>
                </div>

                <button class="start-game-btn" @click="startGame" :disabled="players.length < 2">
                    START GAME
                </button>
            </div>
        </div>

        <Footer />
    </div>
</template>

<style scoped>
.lobby-screen {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background-color: #000;
    background-image: radial-gradient(circle at 50% 50%, #1a0033 0%, #000 100%);
    color: white;
    display: flex;
    justify-content: center;
    align-items: center;
    font-family: 'Segoe UI', sans-serif;
}

.overlay {
    position: absolute;
    width: 100%;
    height: 100%;
    background: url('https://www.transparenttextures.com/patterns/dark-matter.png');
    opacity: 0.1;
    pointer-events: none;
}

.content-container {
    z-index: 10;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(10px);
    padding: 40px;
    border-radius: 20px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    text-align: center;
    width: 600px;
    box-shadow: 0 0 30px rgba(138, 43, 226, 0.2);
}

.lobby-title {
    font-size: 3rem;
    font-weight: 900;
    letter-spacing: 5px;
    margin-bottom: 20px;
    text-shadow: 0 0 10px rgba(138, 43, 226, 0.8);
}

.actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 20px;
}

.action-box {
    flex: 1;
    background: rgba(255, 255, 255, 0.05);
    padding: 20px;
    border-radius: 10px;
}

.divider {
    font-weight: bold;
    color: #888;
}

.code-input {
    background: rgba(0, 0, 0, 0.5);
    border: 1px solid #444;
    color: white;
    padding: 10px;
    border-radius: 5px;
    font-size: 1.2rem;
    text-align: center;
    width: 100%;
    margin-bottom: 15px;
    text-transform: uppercase;
    letter-spacing: 3px;
}

.action-btn {
    width: 100%;
    padding: 12px;
    background: linear-gradient(45deg, #8a2be2, #4b0082);
    border: none;
    color: white;
    font-weight: bold;
    cursor: pointer;
    border-radius: 5px;
    transition: transform 0.2s;
}

.action-btn:hover {
    transform: scale(1.05);
}

.error-msg {
    color: #ff4444;
    margin-bottom: 15px;
    background: rgba(255, 0, 0, 0.1);
    padding: 10px;
    border-radius: 5px;
}

.room-header {
    margin-bottom: 30px;
    font-size: 1.5rem;
}

.code-display {
    font-family: monospace;
    font-weight: bold;
    color: #8a2be2;
    margin-left: 10px;
    font-size: 2rem;
    letter-spacing: 2px;
}

.players-list ul {
    list-style: none;
    padding: 0;
    text-align: left;
}

.players-list li {
    padding: 10px;
    background: rgba(255, 255, 255, 0.05);
    margin-bottom: 5px;
    border-radius: 5px;
    display: flex;
    justify-content: space-between;
}

.players-list li.self {
    border: 1px solid #8a2be2;
}

.char-bar {
    display: flex;
    justify-content: center;
    gap: 20px;
    margin-top: 20px;
}

.start-game-btn {
    margin-top: 30px;
    padding: 15px 50px;
    font-size: 1.2rem;
    background: linear-gradient(to right, #00c6ff, #0072ff);
    border: none;
    color: white;
    font-weight: 900;
    border-radius: 50px;
    cursor: pointer;
    box-shadow: 0 0 20px rgba(0, 114, 255, 0.4);
    transition: all 0.3s;
}

.start-game-btn:disabled {
    background: #555;
    cursor: not-allowed;
    box-shadow: none;
    opacity: 0.7;
}

.start-game-btn:not(:disabled):hover {
    transform: translateY(-2px);
    box-shadow: 0 5px 25px rgba(0, 114, 255, 0.6);
}
</style>
