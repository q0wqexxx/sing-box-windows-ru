// Определение типов сообщений WebSocket
interface WSTrafficData {
  up: number
  down: number
}

interface WSMemoryData {
  inuse: number
  oslimit: number
}

interface WSLogData {
  time: string
  level: string
  message: string
}

type WSData = WSTrafficData | WSMemoryData | WSLogData

export const createWebSocket = (
  url: string,
  onMessage: (data: WSData) => void,
  onClose?: () => void,
) => {
  if (typeof WebSocket === 'undefined') {
    alert('Ваш браузер не поддерживает WebSocket')
    return
  }

  let ws: WebSocket | null = null
  let reconnectTimer: number | null = null

  const connect = () => {
    ws = new WebSocket(url)

    ws.onerror = () => {
      console.log('Ошибка подключения WebSocket')
      onClose?.()
      scheduleReconnect()
    }

    ws.onopen = () => {
      console.log('Подключение WebSocket успешно')
      if (reconnectTimer) {
        window.clearTimeout(reconnectTimer)
        reconnectTimer = null
      }
    }

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data) as WSData
        onMessage(data)
      } catch (error) {
        console.error('Ошибка разбора сообщения:', error)
      }
    }

    ws.onclose = () => {
      console.log('Подключение WebSocket закрыто')
      onClose?.()
      scheduleReconnect()
    }
  }

  const scheduleReconnect = () => {
    if (!reconnectTimer) {
      reconnectTimer = window.setTimeout(() => {
        console.log('Попытка переподключения...')
        connect()
      }, 3000) // Переподключение через 3 секунды
    }
  }

  connect()

  // Возвращаем функцию очистки
  return () => {
    if (ws) {
      ws.close()
    }
    if (reconnectTimer) {
      window.clearTimeout(reconnectTimer)
    }
  }
}

export function formatBandwidth(kbps: number) {
  kbps = kbps / 1024
  // Рассчитываем MB/s и GB/s
  const mbps = kbps / 1024 // Преобразование KB/s в MB/s
  const gbps = mbps / 1024 // Преобразование MB/s в GB/s

  // Выбираем наилучшую единицу измерения
  let formattedBandwidth
  if (gbps >= 1) {
    formattedBandwidth = `${gbps.toFixed(2)} GB`
  } else if (mbps >= 1) {
    formattedBandwidth = `${mbps.toFixed(2)} MB`
  } else {
    formattedBandwidth = `${kbps.toFixed(2)} KB`
  }

  // Форматируем вывод, оставляя две цифры после запятой
  return formattedBandwidth
}
