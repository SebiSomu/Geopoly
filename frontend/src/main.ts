import { createApp } from 'vue'
import './style.css'
import App from './App.vue'
import router from './router'
import { setupApollo } from './apollo'

setupApollo()

const app = createApp(App)
app.use(router)
app.mount('#app')
