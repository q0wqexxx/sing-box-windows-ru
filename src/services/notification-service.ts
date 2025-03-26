import { MessageType } from '@/stores/infoStore'
import { useInfoStore } from '@/stores/infoStore'

/**
 * Сервис уведомлений, предоставляющий единый функционал для уведомлений
 */
export class NotificationService {
  private static instance: NotificationService
  private messageHandler: ((type: MessageType, content: string) => void) | null = null
  private infoStore = useInfoStore()

  private constructor() {
    // Приватный конструктор, предотвращающий создание экземпляра извне
  }

  /**
   * Получение единственного экземпляра NotificationService
   */
  public static getInstance(): NotificationService {
    if (!NotificationService.instance) {
      NotificationService.instance = new NotificationService()
    }
    return NotificationService.instance
  }

  /**
   * Установка обработчика сообщений
   * @param handler Функция обратного вызова для обработки сообщений
   */
  public setMessageHandler(handler: (type: MessageType, content: string) => void): void {
    this.messageHandler = handler
    this.infoStore.setMessageCallback(handler)
  }

  /**
   * Показ успешного сообщения
   * @param content Содержание сообщения
   */
  public success(content: string): void {
    this.showMessage('success', content)
  }

  /**
   * Показ информационного сообщения
   * @param content Содержание сообщения
   */
  public info(content: string): void {
    this.showMessage('info', content)
  }

  /**
   * Показ предупреждающего сообщения
   * @param content Содержание сообщения
   */
  public warning(content: string): void {
    this.showMessage('warning', content)
  }

  /**
   * Показ сообщения об ошибке
   * @param content Содержание сообщения
   */
  public error(content: string): void {
    this.showMessage('error', content)
  }

  /**
   * Общий метод для показа сообщений
   * @param type Тип сообщения
   * @param content Содержание сообщения
   */
  private showMessage(type: MessageType, content: string): void {
    if (this.messageHandler) {
      this.messageHandler(type, content)
    } else {
      // Если обработчик не установлен, используем обработчик по умолчанию из InfoStore
      this.infoStore.showMessage(type, content)
    }
  }
}
