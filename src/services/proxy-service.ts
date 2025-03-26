import { Window } from '@tauri-apps/api/window'
import { useAppStore } from '@/stores/AppStore'
import { useInfoStore } from '@/stores/infoStore'
import { tauriApi } from './tauri-api'
import { NotificationService } from './notification-service'

export class ProxyService {
  private static instance: ProxyService
  private appStore = useAppStore()
  private infoStore = useInfoStore()
  private notificationService = NotificationService.getInstance()

  private constructor() {}

  public static getInstance(): ProxyService {
    if (!ProxyService.instance) {
      ProxyService.instance = new ProxyService()
    }
    return ProxyService.instance
  }

  /**
   * Переключение режима прокси
   * @param mode Режим прокси
   * @param showMessage Функция для показа сообщений (опционально)
   * @returns Нужно ли закрыть окно
   */
  public async switchMode(
    mode: 'system' | 'tun',
    showMessage?: (type: 'success' | 'info' | 'error', content: string) => void,
  ): Promise<boolean> {
    try {
      if (mode === 'system') {
        await tauriApi.proxy.setSystemProxy()
        this.appStore.proxyMode = 'system'
        if (showMessage) showMessage('success', 'Переключено на системный прокси режим')
        else this.notificationService.success('Переключено на системный прокси режим')
      } else {
        // Режим TUN требует прав администратора
        const isAdmin = await tauriApi.proxy.checkAdmin()
        if (!isAdmin) {
          try {
            await tauriApi.proxy.restartAsAdmin()
            return true // Нужно закрыть окно
          } catch (error) {
            if (showMessage) showMessage('error', 'Не удалось получить права администратора')
            else this.notificationService.error('Не удалось получить права администратора')
            return false
          }
        }
        await tauriApi.proxy.setTunProxy()
        this.appStore.proxyMode = 'tun'
        if (showMessage) showMessage('success', 'Переключено на режим TUN')
        else this.notificationService.success('Переключено на режим TUN')
      }

      // Если ядро работает, нужно перезапустить
      if (this.appStore.isRunning) {
        try {
          if (showMessage) showMessage('info', 'Перезапуск ядра...')
          else this.notificationService.info('Перезапуск ядра...')

          await this.infoStore.restartKernel()

          if (showMessage) showMessage('success', 'Ядро перезапущено')
          else this.notificationService.success('Ядро перезапущено')
        } catch (error) {
          const errorMsg = `Не удалось перезапустить ядро: ${error}`
          if (showMessage) showMessage('error', errorMsg)
          else this.notificationService.error(errorMsg)
        }
      }

      return false // Не нужно закрывать окно
    } catch (error) {
      const errorMsg = `Не удалось переключить режим прокси: ${error}`
      if (showMessage) showMessage('error', errorMsg)
      else this.notificationService.error(errorMsg)
      return false
    }
  }
}
