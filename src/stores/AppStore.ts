import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { tauriApi } from '@/services/tauri-api'
import { getVersion } from '@tauri-apps/api/app'
import { darkTheme } from 'naive-ui'
import { useOsTheme } from 'naive-ui'
import { Window } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'
import mitt from '@/utils/mitt'
import { useRouter, Router } from 'vue-router'

// Определение типа информации об обновлении
interface UpdateInfo {
  latest_version: string
  download_url: string
  has_update: boolean
}

// Тип состояния окна
export interface WindowState {
  isVisible: boolean
  isFullscreen: boolean
  lastVisiblePath: string
}

export const useAppStore = defineStore(
  'app',
  () => {
    // Состояние работы приложения
    const isRunning = ref(false)

    // ID экземпляра трея - используется TrayStore
    const trayInstanceId = ref<string | null>(null)

    // Режим прокси
    const proxyMode = ref<'system' | 'tun'>('system')

    // Настройки автозапуска
    const autoStartApp = ref(false)
    const autoStartKernel = ref(false)

    // Настройки версии IP
    const preferIpv6 = ref(false)

    // Состояние версии приложения
    const appVersion = ref('')

    // Состояние обновлений
    const hasUpdate = ref(false)
    const latestVersion = ref('')
    const downloadUrl = ref('')

    // Состояние окна
    const windowState = ref<WindowState>({
      isVisible: true,
      isFullscreen: false,
      lastVisiblePath: '/',
    })

    // Состояние темы
    const osTheme = useOsTheme()
    const isDark = ref(osTheme.value === 'dark')
    const theme = computed(() => (isDark.value ? darkTheme : null))

    // Переключение темы
    const toggleTheme = () => {
      isDark.value = !isDark.value
    }

    // Получение версии приложения
    const fetchAppVersion = async () => {
      try {
        appVersion.value = await getVersion()
      } catch (error) {
        console.error('Не удалось получить версию приложения:', error)
      }
    }

    // Проверка обновлений
    const checkUpdate = async (silent: boolean = false): Promise<UpdateInfo | null> => {
      try {
        const updateInfo = await tauriApi.update.checkUpdate(appVersion.value)

        if (updateInfo && updateInfo.has_update) {
          hasUpdate.value = true
          latestVersion.value = updateInfo.latest_version
          downloadUrl.value = updateInfo.download_url

          // Уведомление только в не тихом режиме
          if (!silent) {
            mitt.emit('update-available', updateInfo)
          }

          return updateInfo
        }

        return null
      } catch (error) {
        console.error('Не удалось проверить обновления:', error)
        return null
      }
    }

    // Загрузка и установка обновлений
    const downloadAndInstallUpdate = async () => {
      if (!hasUpdate.value || !downloadUrl.value) return false

      try {
        // Уведомление о начале загрузки
        mitt.emit('download-progress', {
          status: 'checking',
          progress: 0,
          message: 'Подготовка к загрузке обновления...',
        })

        // Начало загрузки и установки
        const result = await tauriApi.update.downloadAndInstallUpdate(downloadUrl.value)
        return result
      } catch (error) {
        console.error('Не удалось загрузить обновление:', error)
        return false
      }
    }

    // Изменение состояния работы приложения
    const setRunningState = (state: boolean) => {
      if (isRunning.value !== state) {
        isRunning.value = state
        // Отправка события изменения состояния процесса
        mitt.emit('process-status')
      }
    }

    // Переключение режима прокси
    const switchProxyMode = async (targetMode: 'system' | 'tun') => {
      // Если текущий режим совпадает с целевым, переключение не требуется
      if (proxyMode.value === targetMode) return

      // Вызов соответствующего сервиса в зависимости от режима
      try {
        if (targetMode === 'system') {
          await tauriApi.proxy.setSystemProxy()
        } else {
          // Режим TUN может требовать прав администратора, проверка и обработка
          const isAdmin = await tauriApi.proxy.checkAdmin()
          if (!isAdmin) {
            // Требуются права администратора, перезапуск
            await tauriApi.proxy.restartAsAdmin()
            return
          }
          await tauriApi.proxy.setTunProxy()
        }

        // Обновление состояния после успешного переключения
        proxyMode.value = targetMode

        // Отправка события изменения режима прокси, уведомление других компонентов
        mitt.emit('proxy-mode-changed')
      } catch (error) {
        console.error('Не удалось переключить режим прокси:', error)
        throw error
      }
    }

    // Получение текущего окна приложения
    const getAppWindow = () => Window.getCurrent()

    // Минимизация окна
    const minimizeWindow = async () => {
      const appWindow = getAppWindow()
      await appWindow.minimize()
      // Отправка события минимизации
      mitt.emit('window-minimize')
    }

    // Скрытие окна
    const hideWindow = async () => {
      const appWindow = getAppWindow()
      await appWindow.hide()
      windowState.value.isVisible = false
      // Отправка события скрытия
      mitt.emit('window-hide')
    }

    // Показ окна
    const showWindow = async () => {
      const appWindow = getAppWindow()
      await appWindow.show()
      await appWindow.setFocus()
      windowState.value.isVisible = true
      // Отправка события показа
      mitt.emit('window-show')
    }

    // Установка окна поверх всех окон
    const setWindowAlwaysOnTop = async () => {
      const appWindow = getAppWindow()
      await appWindow.setAlwaysOnTop(true)
    }

    // Получение видимости окна
    const getWindowVisible = async () => {
      const appWindow = getAppWindow()
      return await appWindow.isVisible()
    }

    // Переключение полноэкранного режима
    const toggleFullScreen = async () => {
      const appWindow = getAppWindow()
      const isFullscreen = await appWindow.isFullscreen()

      if (isFullscreen) {
        await appWindow.setFullscreen(false)
      } else {
        await appWindow.setFullscreen(true)
      }

      windowState.value.isFullscreen = !isFullscreen
    }

    // Сохранение состояния маршрута и переход на пустую страницу
    const saveRouteAndGoBlank = (router: Router) => {
      windowState.value.lastVisiblePath = router.currentRoute.value.path
      if (windowState.value.lastVisiblePath !== '/blank') {
        router.push('/blank')
      }
    }

    // Восстановление маршрута с пустой страницы
    const restoreFromBlank = (router: Router) => {
      if (router.currentRoute.value.path === '/blank' && windowState.value.lastVisiblePath) {
        router.push(windowState.value.lastVisiblePath)
      }
    }

    // Настройка обработчиков событий окна
    const setupWindowEventHandlers = (router: Router) => {
      // Переключение на пустую страницу при скрытии окна
      mitt.on('window-hide', () => {
        saveRouteAndGoBlank(router)
      })

      // Восстановление маршрута при показе окна
      mitt.on('window-show', () => {
        restoreFromBlank(router)
      })

      // Восстановление маршрута при восстановлении окна
      mitt.on('window-restore', () => {
        restoreFromBlank(router)
      })

      // Проверка текущего состояния окна
      getAppWindow()
        .isVisible()
        .then((visible) => {
          windowState.value.isVisible = visible
          if (visible) {
            restoreFromBlank(router)
          }
        })
    }

    // Очистка событий окна
    const cleanupWindowEvents = () => {
      mitt.off('window-minimize')
      mitt.off('window-hide')
      mitt.off('window-show')
      mitt.off('window-restore')
    }

    return {
      trayInstanceId,
      isRunning,
      proxyMode,
      autoStartApp,
      autoStartKernel,
      preferIpv6,
      switchProxyMode,
      appVersion,
      fetchAppVersion,
      // Экспорт методов и состояния обновлений
      hasUpdate,
      latestVersion,
      downloadUrl,
      checkUpdate,
      downloadAndInstallUpdate,
      // Экспорт состояния и методов темы
      isDark,
      theme,
      toggleTheme,
      // Состояние окна
      windowState,
      // Операции с окном
      minimizeWindow,
      hideWindow,
      showWindow,
      toggleFullScreen,
      getWindowVisible,
      setWindowAlwaysOnTop,
      // Обработка событий окна
      saveRouteAndGoBlank,
      restoreFromBlank,
      setupWindowEventHandlers,
      cleanupWindowEvents,
      // Обновление состояния работы
      setRunningState,
    }
  },
  {
    persist: true, // Использование конфигурации по умолчанию для сохранения состояния
  },
)
