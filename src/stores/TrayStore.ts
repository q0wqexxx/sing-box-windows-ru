// TrayStore.ts - Store для управления функциями трея приложения
import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { TrayIcon, TrayIconEvent } from '@tauri-apps/api/tray'
import { defaultWindowIcon } from '@tauri-apps/api/app'
import { Menu } from '@tauri-apps/api/menu'
import { MenuItem, type MenuItemOptions } from '@tauri-apps/api/menu/menuItem'
import { Submenu, type SubmenuOptions } from '@tauri-apps/api/menu/submenu'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from './AppStore'
import { useInfoStore } from './infoStore'
import { useSubStore } from './SubStore'
import { ProxyService } from '@/services/proxy-service'
import mitt from '@/utils/mitt'
import { Window } from '@tauri-apps/api/window'
import { useRouter } from 'vue-router'

// Объявление типов событий mitt
declare module '@/utils/mitt' {
  interface Events {
    'refresh-tray-menu': void
    'process-status': void
    'proxy-mode-changed': void
  }
}

// Определение типа пользовательских элементов меню
export interface TrayMenuOptions {
  type?: 'normal' | 'separator' | 'checkbox'
  id: string
  text: string
  checked?: boolean
  enabled?: boolean
  action?: () => Promise<void>
  children?: TrayMenuOptions[]
}

// Определение типа элементов меню Tauri
interface TauriMenuItem {
  id?: string
  text?: string
  type?: string
  checked?: boolean
  enabled?: boolean
  action?: () => Promise<void>
  submenu?: TauriMenuItem[]
}

export const useTrayStore = defineStore(
  'tray',
  () => {
    // Ссылки на другие Store
    const appStore = useAppStore()
    const infoStore = useInfoStore()
    const subStore = useSubStore()
    const router = useRouter()
    const proxyService = ProxyService.getInstance()

    // Добавление внутреннего состояния для записи режима прокси при последнем обновлении меню
    const lastProxyMode = ref<'system' | 'tun'>(appStore.proxyMode)

    // Хранение только ID трея, не экземпляра
    const trayInstanceId = ref<string | null>(null)

    /**
     * Обновление подсказки трея
     */
    const updateTrayTooltip = async () => {
      if (trayInstanceId.value) {
        const status = appStore.isRunning ? 'Работает' : 'Остановлено'
        const mode = appStore.proxyMode === 'system' ? 'Системный прокси' : 'Режим TUN'

        // Получение текущего имени конфигурации
        let configName = ''
        if (subStore.activeIndex !== null && subStore.list.length > 0) {
          configName = subStore.list[subStore.activeIndex].name
        }

        // Составление текста подсказки
        let tooltipText = `sing-box-window - Ядро ${status}, ${mode}`

        // Если есть имя конфигурации, отображаем его
        if (configName) {
          tooltipText += `, Конфигурация: ${configName}`
        }
        try {
          const tray = await TrayIcon.getById(trayInstanceId.value)
          if (tray) {
            await tray.setTooltip(tooltipText)
          }
        } catch (e) {
          console.error('Не удалось обновить подсказку трея:', e)
        }
      }
    }

    /**
     * Создание меню трея
     */
    const createTrayMenu = async () => {
      try {
        // Синхронизация текущего режима прокси, чтобы меню использовало актуальное состояние
        const currentProxyMode = appStore.proxyMode

        // Обновление режима прокси при последнем обновлении меню
        lastProxyMode.value = currentProxyMode

        console.log(`Создание меню трея, текущий режим прокси: ${currentProxyMode}`)

        // Создание основных элементов меню
        const showMenuItem = await MenuItem.new({
          id: 'show',
          text: 'Показать интерфейс',
          enabled: true,
          action: async () => {
            await appStore.restoreFromBlank(router)
            await appStore.showWindow()
          },
        })

        // Создание подменю управления ядром
        const startMenuItem = await MenuItem.new({
          id: 'start',
          text: 'Запустить ядро',
          enabled: !appStore.isRunning,
          action: async () => {
            try {
              await infoStore.startKernel()
              appStore.setRunningState(true)
              await refreshTrayMenu() // Обновление меню для обновления состояния
            } catch (error) {
              console.error('Не удалось запустить ядро:', error)
            }
          },
        })

        const stopMenuItem = await MenuItem.new({
          id: 'stop',
          text: 'Остановить ядро',
          enabled: appStore.isRunning,
          action: async () => {
            await infoStore.stopKernel()
            appStore.setRunningState(false)
            await refreshTrayMenu() // Обновление меню для обновления состояния
          },
        })

        const restartMenuItem = await MenuItem.new({
          id: 'restart',
          text: 'Перезапустить ядро',
          enabled: appStore.isRunning,
          action: async () => {
            await infoStore.restartKernel()
            await refreshTrayMenu() // Обновление меню для обновления состояния
          },
        })

        // Создание подменю управления ядром
        const kernelSubmenu = await Submenu.new({
          id: 'kernel_control',
          text: 'Управление ядром',
          items: [startMenuItem, stopMenuItem, restartMenuItem],
        })

        // Создание подменю режима прокси - использование обычного MenuItem вместо CheckMenuItem
        const systemProxyMenuItem = await MenuItem.new({
          id: 'system_proxy',
          text: 'Системный прокси режим',
          // Отключение кнопки, если текущий режим - системный прокси
          enabled: currentProxyMode !== 'system',
          action: async () => {
            try {
              console.log('Переключение на системный прокси режим')
              await proxyService.switchMode('system')
              appStore.proxyMode = 'system'
              // Принудительное немедленное обновление меню
              await refreshTrayMenu()
            } catch (error) {
              console.error('Не удалось переключить на системный прокси режим:', error)
            }
          },
        })

        const tunProxyMenuItem = await MenuItem.new({
          id: 'tun_mode',
          text: 'Режим TUN',
          // Отключение кнопки, если текущий режим - TUN
          enabled: currentProxyMode !== 'tun',
          action: async () => {
            try {
              console.log('Переключение на режим TUN')
              const needClose = await proxyService.switchMode('tun')
              appStore.proxyMode = 'tun'
              // Принудительное немедленное обновление меню
              await refreshTrayMenu()
              if (needClose) {
                const appWindow = Window.getCurrent()
                await appWindow.close()
              }
            } catch (error) {
              console.error('Не удалось переключить на режим TUN:', error)
            }
          },
        })

        // Текущий режим (только для отображения, не кликабельно)
        const currentModeMenuItem = await MenuItem.new({
          id: 'current_mode',
          text: `Текущий режим: ${currentProxyMode === 'system' ? 'Системный прокси' : 'Режим TUN'}`,
          enabled: false,
        })

        // Создание подменю режима прокси
        const proxyModeSubmenu = await Submenu.new({
          id: 'proxy_mode',
          text: 'Режим прокси',
          items: [currentModeMenuItem, systemProxyMenuItem, tunProxyMenuItem],
        })

        // Создание разделителей меню
        const separator1 = await MenuItem.new({
          id: 'separator1',
          text: '-',
          enabled: false,
        })

        const separator2 = await MenuItem.new({
          id: 'separator2',
          text: '-',
          enabled: false,
        })

        // Создание элемента меню выхода
        const quitMenuItem = await MenuItem.new({
          id: 'quit',
          text: 'Выход',
          action: async () => {
            await infoStore.stopKernel()
            const appWindow = Window.getCurrent()
            await appWindow.close()
          },
        })

        // Создание основного меню
        return await Menu.new({
          items: [
            showMenuItem,
            separator1,
            kernelSubmenu,
            proxyModeSubmenu,
            separator2,
            quitMenuItem,
          ],
        })
      } catch (error) {
        console.error('Не удалось создать меню:', error)
        // Возвращение пустого меню в случае ошибки
        return await Menu.new({ items: [] })
      }
    }

    /**
     * Инициализация трея
     */
    const initTray = async () => {
      try {
        // Очистка предыдущего экземпляра трея (если существует)
        if (appStore.trayInstanceId) {
          try {
            await TrayIcon.removeById(appStore.trayInstanceId)
          } catch (error) {
            // Игнорирование возможных ошибок
          }
        }

        // Создание меню
        const menu = await createTrayMenu()

        // Установка иконки трея
        const icon = await defaultWindowIcon()

        // Убедитесь, что иконка не null
        if (!icon) {
          throw new Error('Не удалось получить иконку окна по умолчанию')
        }

        const options = {
          icon, // Использование не-null иконки
          tooltip: 'sing-box-window', // Инициализация подсказки
          action: async (event: TrayIconEvent) => {
            switch (event.type) {
              case 'Click':
                // Если клик левой кнопкой, показать интерфейс
                if (event.button === 'Left') {
                  await appStore.restoreFromBlank(router)
                  await appStore.showWindow()
                }
                break
            }
          },
          menu,
          menuOnLeftClick: false,
        }

        try {
          // Создание экземпляра трея, но хранение только ID
          const trayInstance = await TrayIcon.new(options)
          trayInstanceId.value = trayInstance.id
          appStore.trayInstanceId = trayInstance.id
        } catch (error) {
          console.error('Не удалось создать экземпляр трея:', error)
          throw error
        }

        // Инициализация текста подсказки
        await updateTrayTooltip()

        // Слушатели изменений состояния для обновления подсказки и меню
        watch(
          () => appStore.isRunning,
          () => {
            updateTrayTooltip()
            refreshTrayMenu() // Обновление меню при изменении состояния работы
          },
        )

        // Слушатель изменений режима прокси и принудительное обновление меню
        watch(
          () => appStore.proxyMode,
          (newMode) => {
            console.log(`Режим прокси изменен на: ${newMode}, предыдущий режим: ${lastProxyMode.value}`)
            updateTrayTooltip()
            // Принудительное обновление меню, если режим действительно изменился
            if (newMode !== lastProxyMode.value) {
              console.log('Режим изменился, принудительное обновление меню трея')
              refreshTrayMenu()
            }
          },
        )

        watch(() => [subStore.activeIndex, subStore.list.length], updateTrayTooltip)

        // Добавление обработчиков событий
        mitt.on('process-status', () => {
          updateTrayTooltip()
          refreshTrayMenu() // Обновление меню при изменении состояния процесса
        })

        mitt.on('proxy-mode-changed', () => {
          console.log('Получено событие изменения режима прокси, обновление меню трея')
          updateTrayTooltip()
          refreshTrayMenu() // Обновление меню при изменении режима прокси
        })

        // Слушатель события обновления меню
        mitt.on('refresh-tray-menu', () => {
          console.log('Получено событие обновления меню трея')
          refreshTrayMenu()
        })

        return true
      } catch (error) {
        console.error('Не удалось инициализировать трей:', error)
        return false
      }
    }

    /**
     * Обновление меню трея
     */
    const refreshTrayMenu = async () => {
      if (!trayInstanceId.value) return

      // Использование метода экземпляра TrayIcon
      try {
        const tray = await TrayIcon.getById(trayInstanceId.value)
        const menu = await createTrayMenu()
        if (tray) {
          await tray.setMenu(menu)
          await updateTrayTooltip()
          console.log('Успешное обновление меню с использованием экземпляра TrayIcon')
        } else {
          throw new Error('Не удалось получить экземпляр трея')
        }
      } catch (trayError) {
        console.error('Не удалось установить меню с использованием экземпляра трея:', trayError)

        // Если все еще не удалось, последний способ - пересоздать трей
        await destroyTray()
        await initTray()
      }
    }

    /**
     * Очистка ресурсов трея
     */
    const destroyTray = async () => {
      if (trayInstanceId.value) {
        try {
          // Использование статического метода для удаления трея
          await TrayIcon.removeById(trayInstanceId.value)
          trayInstanceId.value = null
          appStore.trayInstanceId = null
        } catch (error) {
          console.error('Не удалось удалить трей:', error)
        }
      }

      // Удаление обработчиков событий
      mitt.off('process-status')
      mitt.off('proxy-mode-changed')
      mitt.off('refresh-tray-menu')
    }

    return {
      trayInstanceId,
      initTray,
      updateTrayTooltip,
      refreshTrayMenu,
      destroyTray,
    }
  },
  {
    persist: true,
  },
)
