<template>
  <n-config-provider :theme="appStore.theme" :theme-overrides="themeOverrides">
    <n-dialog-provider>
      <n-modal-provider>
        <n-notification-provider>
          <n-message-provider>
            <router-view />
          </n-message-provider>
        </n-notification-provider>
      </n-modal-provider>
    </n-dialog-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import themeOverrides from '@/assets/naive-ui-theme-overrides.json'
import { onMounted, onUnmounted } from 'vue'
import { useAppStore } from '@/stores/AppStore'
import { useInfoStore } from '@/stores/infoStore'
import { useTrayStore } from '@/stores/TrayStore'
import { useRouter } from 'vue-router'
import { Window } from '@tauri-apps/api/window'
import mitt from '@/utils/mitt'

// Инициализация store
const appStore = useAppStore()
const infoStore = useInfoStore()
const trayStore = useTrayStore()
const router = useRouter()

onMounted(async () => {
  // Установка обработчиков событий окна
  // appStore.setupWindowEventHandlers(router)

  // Реализация собственных обработчиков событий окна
  // Переключение на пустую страницу при скрытии окна
  mitt.on('window-hide', () => {
    appStore.windowState.lastVisiblePath = router.currentRoute.value.path
    if (appStore.windowState.lastVisiblePath !== '/blank') {
      router.push('/blank')
    }
  })

  // Восстановление маршрута при показе окна
  mitt.on('window-show', () => {
    if (router.currentRoute.value.path === '/blank' && appStore.windowState.lastVisiblePath) {
      router.push(appStore.windowState.lastVisiblePath)
    }
  })

  // Восстановление маршрута при восстановлении окна
  mitt.on('window-restore', () => {
    if (router.currentRoute.value.path === '/blank' && appStore.windowState.lastVisiblePath) {
      router.push(appStore.windowState.lastVisiblePath)
    }
  })

  // Проверка текущего состояния окна
  const appWindow = Window.getCurrent()
  appWindow.isVisible().then((visible) => {
    appStore.windowState.isVisible = visible
    if (
      visible &&
      router.currentRoute.value.path === '/blank' &&
      appStore.windowState.lastVisiblePath
    ) {
      router.push(appStore.windowState.lastVisiblePath)
    }
  })

  // Инициализация значка трея
  await trayStore.initTray()

  // Если не в режиме разработки, отключить контекстное меню
  if (!import.meta.env.DEV) {
    document.oncontextmenu = () => false
  }

  // Если включен автозапуск ядра
  if (appStore.autoStartKernel) {
    try {
      await infoStore.startKernel()
      appStore.setRunningState(true)

      // Проверка, нужно ли скрыть окно
      const appWindow = Window.getCurrent()
      if (!(await appWindow.isVisible())) {
        appStore.saveRouteAndGoBlank(router)
      }
    } catch (error) {
      console.error('Не удалось автоматически запустить ядро:', error)
    }
  }
  // Если ядро уже запущено, инициализировать WebSocket соединение
  if (appStore.isRunning) {
    infoStore.initEventListeners()
  }
})

onUnmounted(async () => {
  // Очистка обработчиков событий
  // appStore.cleanupWindowEvents()
  mitt.off('window-minimize')
  mitt.off('window-hide')
  mitt.off('window-show')
  mitt.off('window-restore')

  // Удаление значка трея
  await trayStore.destroyTray()
})
</script>

<style>
#app {
  height: 100vh;
}
</style>
