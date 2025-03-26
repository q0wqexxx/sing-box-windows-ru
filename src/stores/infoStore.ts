import { defineStore } from 'pinia'
import { ref, onUnmounted } from 'vue'
import { tauriApi } from '@/services/tauri-api'
import { listen } from '@tauri-apps/api/event'

// Определение типа сообщения
export type MessageType = 'success' | 'info' | 'error' | 'warning'

// Определение интерфейса информации о версии
interface VersionInfo {
  version: string
  meta: boolean
  premium: boolean
  environment?: string
  tags?: string[]
  revision?: string
  cgo?: string
}

// Определение интерфейса данных о соединении
interface ConnectionMetadata {
  destinationIP: string
  destinationPort: string
  dnsMode: string
  host: string
  network: string
  processPath: string
  sourceIP: string
  sourcePort: string
  type: string
}

interface Connection {
  chains: string[]
  download: number
  id: string
  metadata: ConnectionMetadata
  rule: string
  rulePayload: string
  start: string
  upload: number
}

interface ConnectionsData {
  connections: Connection[]
  downloadTotal: number
  uploadTotal: number
  memory: number
}

export const useInfoStore = defineStore(
  'info',
  () => {
    // Информация о версии
    const version = ref<VersionInfo>({ version: '', meta: true, premium: true })
    const newVersion = ref('')

    // Информация об использовании памяти
    const memory = ref({
      inuse: 0,
      oslimit: 0,
    })

    // Информация о трафике
    const traffic = ref({
      up: 0,
      down: 0,
      total: 0,
      totalUp: 0, // Общий объем загруженного трафика
      totalDown: 0, // Общий объем скачанного трафика
    })

    // Информация о соединениях
    const connections = ref<Connection[]>([])
    const connectionsTotal = ref({
      upload: 0,
      download: 0,
    })

    // Время работы приложения (в секундах)
    const uptime = ref(0)
    let uptimeInterval: NodeJS.Timeout | null = null

    // Информация о логах
    // Уменьшение максимального количества логов для снижения нагрузки на память
    const MAX_LOGS = 200

    interface LogEntry {
      type: string
      payload: string
      timestamp: number
    }

    const logs = ref<LogEntry[]>([])

    // Хранение функций очистки событий
    let cleanupFunctions: Array<() => void> = []
    // Флаг наличия активных слушателей событий
    let activeConnections = false

    // Получение последней версии
    const getLatestVersion = async () => {
      try {
        const res = await fetch('https://api.github.com/repos/SagerNet/sing-box/releases/latest')
        const json = await res.json()
        newVersion.value = json.tag_name.replace('v', '')
      } catch (error) {
        console.error('Не удалось получить последнюю версию:', error)
      }
    }

    // Проверка версии ядра
    const checkKernelVersion = async () => {
      try {
        const output = await tauriApi.kernel.checkKernelVersion()
        if (output) {
          const versionInfo: VersionInfo = {
            version: '',
            meta: true,
            premium: true,
          }

          // Разбор вывода версии
          const lines = output.split('\n')
          for (const line of lines) {
            if (line.startsWith('sing-box version')) {
              versionInfo.version = line.split(' ')[2]
            } else if (line.startsWith('Environment:')) {
              versionInfo.environment = line.split(':')[1].trim()
            } else if (line.startsWith('Tags:')) {
              versionInfo.tags = line.split(':')[1].trim().split(',')
            } else if (line.startsWith('Revision:')) {
              versionInfo.revision = line.split(':')[1].trim()
            } else if (line.startsWith('CGO:')) {
              versionInfo.cgo = line.split(':')[1].trim()
            }
          }
          version.value = versionInfo
          return true
        }
        return false
      } catch (error) {
        console.error('Не удалось проверить версию ядра:', error)
        return false
      }
    }

    // Инициализация слушателей событий
    const initEventListeners = async () => {
      // Если уже есть активные соединения, сначала очистим их
      if (activeConnections) {
        cleanupEventListeners()
      }

      // Установка флага активности
      activeConnections = true

      // Начало отсчета времени работы
      uptime.value = 0
      uptimeInterval = setInterval(() => {
        uptime.value += 1
      }, 1000)

      try {
        // Запуск реле WebSocket на стороне сервера
        await tauriApi.kernel.startWebsocketRelay()

        // Слушатель данных о трафике
        const unlistenTraffic = await listen('traffic-data', (event) => {
          const data = event.payload as {
            up: number
            down: number
          }
          if ('up' in data && 'down' in data) {
            const currentUp = Number(data.up) || 0
            const currentDown = Number(data.down) || 0

            // Безопасное обновление общего объема трафика
            const currentTotalUp = Number(traffic.value.totalUp) || 0
            const currentTotalDown = Number(traffic.value.totalDown) || 0

            traffic.value = {
              up: currentUp,
              down: currentDown,
              total: traffic.value.total + currentUp + currentDown,
              totalUp: currentTotalUp + currentUp,
              totalDown: currentTotalDown + currentDown,
            }
          }
        })

        // Слушатель данных о памяти
        const unlistenMemory = await listen('memory-data', (event) => {
          const data = event.payload as {
            inuse: number
            oslimit: number
          }
          if ('inuse' in data && 'oslimit' in data) {
            memory.value = data
          }
        })

        // Слушатель данных логов
        const unlistenLogs = await listen('log-data', (event) => {
          const data = event.payload as {
            type: string
            payload: string
          }
          if (
            'type' in data &&
            'payload' in data &&
            typeof data.type === 'string' &&
            typeof data.payload === 'string'
          ) {
            // Добавление записи лога в начало массива и ограничение максимального количества
            logs.value.unshift({
              type: data.type,
              payload: data.payload,
              timestamp: Date.now(),
            })

            // Удаление лишних логов, если их количество превышает максимум
            if (logs.value.length > MAX_LOGS) {
              logs.value = logs.value.slice(0, MAX_LOGS)
            }
          }
        })

        // Слушатель данных о соединениях
        const unlistenConnections = await listen('connections-data', (event) => {
          const data = event.payload as ConnectionsData
          if ('connections' in data && Array.isArray(data.connections)) {
            connections.value = data.connections

            // Обновление общего объема трафика
            if ('downloadTotal' in data && 'uploadTotal' in data) {
              connectionsTotal.value = {
                download: data.downloadTotal || 0,
                upload: data.uploadTotal || 0,
              }
            }
          }
        })

        // Хранение функций очистки
        cleanupFunctions = [unlistenTraffic, unlistenMemory, unlistenLogs, unlistenConnections]
      } catch (error) {
        console.error('Не удалось инициализировать слушатели событий:', error)
      }
    }

    // Очистка всех слушателей событий
    const cleanupEventListeners = () => {
      if (cleanupFunctions.length > 0) {
        cleanupFunctions.forEach((cleanup) => cleanup())
        cleanupFunctions = []
        activeConnections = false
      }

      // Очистка интервала времени работы
      if (uptimeInterval) {
        clearInterval(uptimeInterval)
        uptimeInterval = null
      }
    }

    // Запуск ядра
    const startKernel = async () => {
      await tauriApi.kernel.startKernel()

      // Сброс всех счетчиков при инициализации
      traffic.value = {
        up: 0,
        down: 0,
        total: 0,
        totalUp: 0,
        totalDown: 0,
      }
      uptime.value = 0
      connections.value = []
      connectionsTotal.value = { upload: 0, download: 0 }

      // Ожидание запуска ядра и проверка состояния
      return new Promise((resolve, reject) => {
        let retryCount = 0
        const maxRetries = 5

        const checkStatus = async () => {
          try {
            // Получение информации о версии с помощью Tauri API
            const json = await tauriApi.proxy.getVersionInfo()
            version.value = json

            // Получение последней версии
            await getLatestVersion()

            // Инициализация слушателей событий
            await initEventListeners()

            resolve(true)
          } catch (error) {
            console.error('Не удалось проверить состояние:', error)
            if (retryCount < maxRetries) {
              retryCount++
              console.log(`Повторная попытка ${retryCount} из ${maxRetries}`)
              setTimeout(checkStatus, 1000)
            } else {
              // Использование значений по умолчанию, если не удалось получить информацию о версии
              console.warn('Не удалось получить информацию о версии, использование значений по умолчанию')
              version.value = { version: 'sing-box неизвестная версия', meta: true, premium: true }

              // Инициализация слушателей событий, даже если не удалось получить информацию о версии
              try {
                await initEventListeners()
                resolve(true)
              } catch (initError) {
                console.error('Не удалось инициализировать слушатели событий:', initError)
                reject(new Error(`Инициализация не удалась: ${initError}`))
              }
            }
          }
        }

        checkStatus()
      })
    }

    // Остановка ядра
    const stopKernel = async () => {
      try {
        await tauriApi.kernel.stopKernel()
      } finally {
        // Очистка слушателей событий и состояния независимо от успеха
        cleanupEventListeners()
        // Сброс состояния
        memory.value = { inuse: 0, oslimit: 0 }
        traffic.value = { up: 0, down: 0, total: 0, totalUp: 0, totalDown: 0 }
        uptime.value = 0
        logs.value = []
        connections.value = []
        connectionsTotal.value = { upload: 0, download: 0 }
      }
    }

    // Перезапуск ядра
    const restartKernel = async () => {
      await stopKernel()
      await startKernel()
    }

    // Обновление информации о версии
    const updateVersion = async () => {
      try {
        await checkKernelVersion()
      } catch (error) {
        console.error('Не удалось получить информацию о версии:', error)
        version.value = { version: '', meta: false, premium: false }
      }
    }

    // Очистка логов
    const clearLogs = () => {
      logs.value = []
    }

    // Функция уведомлений
    let messageCallback: ((type: MessageType, content: string) => void) | null = null

    const setMessageCallback = (callback: (type: MessageType, content: string) => void) => {
      messageCallback = callback
    }

    const showMessage = (type: MessageType, content: string) => {
      if (messageCallback) {
        messageCallback(type, content)
      } else {
        console.log(`[${type}] ${content}`)
      }
    }

    // Переключение версии IP
    const toggleIpVersion = async () => {
      // Реализация логики переключения версии IP
    }

    // Сброс статистики
    const resetStats = () => {
      traffic.value = {
        up: 0,
        down: 0,
        total: 0,
        totalUp: 0,
        totalDown: 0,
      }
      uptime.value = 0
    }

    // Очистка при размонтировании компонента
    onUnmounted(() => {
      cleanupEventListeners()
    })

    return {
      version,
      newVersion,
      memory,
      traffic,
      logs,
      uptime,
      connections,
      connectionsTotal,
      startKernel,
      stopKernel,
      restartKernel,
      initEventListeners,
      updateVersion,
      checkKernelVersion,
      clearLogs,
      cleanupEventListeners,
      // Уведомления
      setMessageCallback,
      showMessage,
      // Операции с ядром
      toggleIpVersion,
      resetStats,
    }
  },
  {
    persist: true,
  },
)
