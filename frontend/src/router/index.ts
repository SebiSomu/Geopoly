import { createRouter, createWebHistory } from 'vue-router'
import StartScreen from '../components/login/StartScreen.vue'
import LobbyView from '../views/LobbyView.vue'
import GameBoard from '../components/board/GameBoard.vue'

const routes = [
    {
        path: '/',
        name: 'Home',
        component: StartScreen
    },
    {
        path: '/lobby',
        name: 'Lobby',
        component: LobbyView
    },
    {
        path: '/game/:code',
        name: 'Game',
        component: GameBoard,
        props: true
    }
]

const router = createRouter({
    history: createWebHistory(),
    routes
})

export default router
