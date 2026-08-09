import DefaultTheme from 'vitepress/theme'
import Landing from './Landing.vue'
import '@fontsource-variable/manrope/wght.css'
import '@fontsource-variable/jetbrains-mono/wght.css'
import './custom.css'

export default {
  extends: DefaultTheme,
  enhanceApp({ app }) {
    app.component('Landing', Landing)
  },
}
