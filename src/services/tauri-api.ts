import { invoke } from '@tauri-apps/api/core'

// Определение типов интерфейсов
interface ProxyData {
  type: string
  name: string
  now: string
  all: string[]
  history: Array<{ time: string; delay: number }>
  udp: boolean
}

interface ProxiesData {
  proxies: Record<string, ProxyData>
}

interface NodeDelay {
  delay: number
}

interface VersionInfo {
  version: string
  meta: boolean
  premium: boolean
  environment?: string
  tags?: string[]
  revision?: string
  cgo?: string
}

// Интерфейсы для управления ядром
export const kernelApi = {
  // Получение использования памяти
  getMemoryUsage: () => invoke<string>('get_memory_usage'),

  // Получение данных о трафике
  getTrafficData: () => invoke<string>('get_traffic_data'),

  // Запуск ядра
  startKernel: () => invoke<void>('start_kernel'),

  // Остановка ядра
  stopKernel: () => invoke<void>('stop_kernel'),

  // Перезапуск ядра
  restartKernel: () => invoke<void>('restart_kernel'),

  // Получение статуса процесса
  getProcessStatus: () => invoke<string>('get_process_status'),

  // Проверка версии ядра
  checkKernelVersion: () => invoke<string>('check_kernel_version'),

  // Запуск реле данных WebSocket
  startWebsocketRelay: () => invoke<void>('start_websocket_relay'),
}

// Интерфейсы для управления прокси
export const proxyApi = {
  // Установка системного прокси режима
  setSystemProxy: () => invoke<void>('set_system_proxy'),

  // Установка режима TUN прокси
  setTunProxy: () => invoke<void>('set_tun_proxy'),

  // Проверка прав администратора
  checkAdmin: () => invoke<boolean>('check_admin'),

  // Перезапуск с правами администратора
  restartAsAdmin: () => invoke<void>('restart_as_admin'),

  // Переключение версии IP
  toggleIpVersion: (preferIpv6: boolean) => invoke<void>('toggle_ip_version', { preferIpv6 }),

  // Переключение режима прокси (global, rule, tun)
  toggleProxyMode: (mode: string) => invoke<string>('toggle_proxy_mode', { mode }),

  // Получение текущего режима прокси
  getCurrentProxyMode: () => invoke<string>('get_current_proxy_mode'),

  // Получение списка прокси
  getProxies: () => invoke<ProxiesData>('get_proxies'),

  // Переключение прокси
  changeProxy: (group: string, proxy: string) => invoke<void>('change_proxy', { group, proxy }),

  // Тестирование задержки узла
  testNodeDelay: (name: string, server?: string) =>
    invoke<NodeDelay>('test_node_delay', { name, server }),

  // Пакетное тестирование задержки узлов
  batchTestNodes: (nodes: string[], server?: string) =>
    invoke<void>('batch_test_nodes', { nodes, server }),

  // Получение информации о версии
  getVersionInfo: () => invoke<VersionInfo>('get_version_info'),

  // Получение списка правил
  getRules: () =>
    invoke<{ rules: Array<{ type: string; payload: string; proxy: string }> }>('get_rules'),
}

// Интерфейсы для управления подписками
export const subscriptionApi = {
  // Загрузка подписки
  downloadSubscription: (url: string) => invoke<void>('download_subscription', { url }),

  // Загрузка последней версии ядра
  downloadLatestKernel: () => invoke<void>('download_latest_kernel'),

  // Получение текущей конфигурации
  getCurrentConfig: () => invoke<string>('get_current_config'),
}

// Экспорт всех API
export const tauriApi = {
  kernel: kernelApi,
  proxy: proxyApi,
  subscription: subscriptionApi,

  // API для обновлений
  update: {
    // Проверка обновлений
    checkUpdate: async (currentVersion: string) => {
      return await invoke<{
        latest_version: string
        download_url: string
        has_update: boolean
      }>('check_update', { currentVersion })
    },

    // Загрузка и установка обновлений
    downloadAndInstallUpdate: async (downloadUrl: string) => {
      return await invoke<void>('download_and_install_update', { downloadUrl })
    },
  },
}
