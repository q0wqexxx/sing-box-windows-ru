<template>
  <div class="proxy-container">
    <!-- Верхняя карточка заголовка -->
    <n-card class="proxy-card" :bordered="false">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <n-h3 class="card-title">
              <n-icon size="24" class="card-icon">
                <swap-horizontal-outline />
              </n-icon>
              Настройки прокси
            </n-h3>
          </div>
          <div class="header-right">
            <!-- Переключение режима прокси -->
            <n-dropdown :options="proxyModeOptions" @select="handleProxyModeChange" trigger="click">
              <n-tooltip trigger="hover" placement="top">
                <template #trigger>
                  <n-tag class="mode-tag" :bordered="false" type="success" size="medium" hoverable>
                    <n-icon size="18" class="mode-icon">
                      <globe-outline v-if="currentProxyMode === 'global'" />
                      <layers-outline v-if="currentProxyMode === 'rule'" />
                      <hardware-chip-outline v-if="currentProxyMode === 'tun'" />
                    </n-icon>
                    {{ getProxyModeText(currentProxyMode) }}
                    <n-icon size="16" class="dropdown-icon">
                      <chevron-down-outline />
                    </n-icon>
                  </n-tag>
                </template>
                Нажмите для переключения режима прокси
              </n-tooltip>
            </n-dropdown>

            <!-- Кнопка обновления -->
            <n-tooltip trigger="hover" placement="top">
              <template #trigger>
                <n-button
                  quaternary
                  circle
                  size="medium"
                  @click="init"
                  :loading="isLoading"
                  class="refresh-button"
                >
                  <template #icon>
                    <n-icon><refresh-outline /></n-icon>
                  </template>
                </n-button>
              </template>
              Обновить список прокси
            </n-tooltip>
          </div>
        </div>
      </template>
    </n-card>

    <!-- Диалог переключения режима прокси -->
    <n-modal
      v-model:show="showModeChangeModal"
      preset="dialog"
      :title="`Переключение на ${targetProxyMode ? getProxyModeText(targetProxyMode) : ''}`"
    >
      <template #header>
        <div class="modal-header">
          <n-icon size="22" class="modal-icon">
            <information-circle-outline />
          </n-icon>
          <span>Переключение на {{ targetProxyMode ? getProxyModeText(targetProxyMode) : '' }}</span>
        </div>
      </template>
      <div class="modal-content">Переключение режима прокси требует перезапуска ядра для вступления в силу. Вы уверены, что хотите переключить и перезапустить ядро?</div>
      <template #action>
        <div class="modal-footer">
          <n-space justify="end">
            <n-button @click="showModeChangeModal = false">Отмена</n-button>
            <n-button type="primary" :loading="isChangingMode" @click="confirmProxyModeChange">
              Подтвердить переключение
            </n-button>
          </n-space>
        </div>
      </template>
    </n-modal>

    <!-- Карточка списка прокси -->
    <n-spin :show="isLoading">
      <n-card class="proxy-list-card" :bordered="false">
        <n-tabs type="segment" animated class="proxy-tabs" v-model:value="activeTab">
          <!-- Вкладка глобальных настроек -->
          <n-tab-pane name="global" tab="Глобальные настройки">
            <div v-if="globalGroup" class="proxy-group">
              <div class="proxy-group-info">
                <n-space align="center" :size="12">
                  <n-tag :bordered="false" type="success" size="medium" class="proxy-tag">
                    Текущий: {{ globalGroup.now }}
                  </n-tag>
                  <n-tag :bordered="false" type="info" size="medium" class="proxy-tag">
                    {{ globalGroup.all.length }} доступных опций
                  </n-tag>
                </n-space>
              </div>

              <n-grid :x-gap="16" :y-gap="16" :cols="gridCols" responsive="screen">
                <n-grid-item v-for="(proxy, i) in globalGroup.all" :key="i">
                  <n-card
                    :class="{
                      'proxy-node-card': true,
                      'proxy-node-card-active': globalGroup.now === proxy,
                    }"
                    :bordered="false"
                    hoverable
                  >
                    <n-space vertical :size="14">
                      <n-flex justify="space-between" align="center">
                        <div class="proxy-name-container">
                          <n-ellipsis style="max-width: 100%" :tooltip="{ width: 'trigger' }">
                            {{ proxy }}
                          </n-ellipsis>
                        </div>
                        <n-tag
                          :type="getProxyTypeColor(getProxyType(proxy))"
                          size="small"
                          :bordered="false"
                          round
                          class="type-tag"
                        >
                          {{ getProxyType(proxy) }}
                        </n-tag>
                      </n-flex>

                      <n-flex justify="space-between" align="center">
                        <n-button
                          @click="changeProxy('GLOBAL', proxy)"
                          :type="globalGroup.now === proxy ? 'default' : 'primary'"
                          size="small"
                          :disabled="globalGroup.now === proxy"
                          :ghost="globalGroup.now !== proxy"
                          class="proxy-button"
                        >
                          <template #icon>
                            <n-icon>
                              <checkmark-circle-outline v-if="globalGroup.now === proxy" />
                              <swap-horizontal-outline v-else />
                            </n-icon>
                          </template>
                          {{ globalGroup.now === proxy ? 'Используется' : 'Переключить' }}
                        </n-button>
                      </n-flex>
                    </n-space>
                  </n-card>
                </n-grid-item>
              </n-grid>
            </div>
          </n-tab-pane>

          <!-- Вкладка групп -->
          <n-tab-pane name="groups" tab="Группы прокси">
            <n-tabs type="line" animated v-model:value="activeGroupTab">
              <n-tab-pane
                v-for="(group, index) in proxyGroups"
                :key="index"
                :name="group.name"
                :tab="group.name"
              >
                <div class="proxy-group">
                  <div class="proxy-group-info">
                    <n-space align="center" :size="12">
                      <n-tag :bordered="false" type="success" size="medium" class="proxy-tag">
                        Текущий узел: {{ group.now }}
                      </n-tag>
                      <n-tag :bordered="false" type="info" size="medium" class="proxy-tag">
                        {{ group.all.length }} узлов
                      </n-tag>
                      <n-tag :bordered="false" type="warning" size="medium" class="proxy-tag">
                        {{ group.type }}
                      </n-tag>
                    </n-space>
                  </div>

                  <n-grid :x-gap="16" :y-gap="16" :cols="gridCols" responsive="screen">
                    <n-grid-item v-for="(proxy, i) in group.all" :key="i">
                      <n-card
                        :class="{
                          'proxy-node-card': true,
                          'proxy-node-card-active': group.now === proxy,
                        }"
                        :bordered="false"
                        hoverable
                      >
                        <n-space vertical :size="14">
                          <n-flex justify="space-between" align="center">
                            <div class="proxy-name-container">
                              <n-ellipsis style="max-width: 100%" :tooltip="{ width: 'trigger' }">
                                {{ proxy }}
                              </n-ellipsis>
                            </div>
                            <n-tag
                              :type="getNodeDelayType(getNodeDelay(proxy))"
                              size="small"
                              :bordered="false"
                              round
                              class="delay-tag"
                            >
                              {{
                                getNodeDelay(proxy) === 0 ? 'Не тестировалось' : getNodeDelay(proxy) + 'мс'
                              }}
                            </n-tag>
                          </n-flex>

                          <n-flex justify="space-between" align="center">
                            <n-button
                              @click="changeProxy(group.name, proxy)"
                              :type="group.now === proxy ? 'default' : 'primary'"
                              size="small"
                              :disabled="group.now === proxy"
                              :ghost="group.now !== proxy"
                              class="proxy-button"
                            >
                              <template #icon>
                                <n-icon>
                                  <checkmark-circle-outline v-if="group.now === proxy" />
                                  <swap-horizontal-outline v-else />
                                </n-icon>
                              </template>
                              {{ group.now === proxy ? 'Используется' : 'Переключить' }}
                            </n-button>
                            <n-button
                              @click="testNodeDelay(proxy)"
                              :loading="testingNodes[proxy]"
                              secondary
                              size="small"
                              type="info"
                              ghost
                              class="proxy-button"
                              v-if="isRealNode(proxy)"
                            >
                              <template #icon>
                                <n-icon><speedometer-outline /></n-icon>
                              </template>
                              Тест скорости
                            </n-button>
                          </n-flex>
                        </n-space>
                      </n-card>
                    </n-grid-item>
                  </n-grid>
                </div>
              </n-tab-pane>
            </n-tabs>
          </n-tab-pane>

          <!-- Вкладка узлов -->
          <n-tab-pane name="nodes" tab="Все узлы">
            <div class="nodes-filter">
              <n-space :size="16" align="center">
                <n-input
                  v-model:value="searchText"
                  placeholder="Поиск по имени узла..."
                  clearable
                  class="search-input"
                >
                  <template #prefix>
                    <n-icon><search-outline /></n-icon>
                  </template>
                </n-input>

                <n-button
                  @click="batchTestAllNodes"
                  :loading="batchTesting"
                  type="primary"
                  ghost
                  size="medium"
                >
                  <template #icon>
                    <n-icon><flash-outline /></n-icon>
                  </template>
                  Тестировать все узлы
                </n-button>
              </n-space>

              <!-- Прогресс бар тестирования -->
              <n-progress
                v-if="batchTesting"
                type="line"
                :percentage="batchTestProgress.percentage"
                :indicator-placement="'inside'"
                class="batch-test-progress"
              >
                {{ batchTestProgress.text }}
              </n-progress>
            </div>

            <n-grid :x-gap="16" :y-gap="16" :cols="gridCols" responsive="screen">
              <n-grid-item v-for="(node, i) in filteredNodes" :key="i">
                <n-card class="proxy-node-card node-card" :bordered="false" hoverable>
                  <n-space vertical :size="14">
                    <n-flex justify="space-between" align="center">
                      <div class="proxy-name-container">
                        <n-ellipsis style="max-width: 100%" :tooltip="{ width: 'trigger' }">
                          {{ node.name }}
                        </n-ellipsis>
                      </div>
                      <n-tag
                        :type="getNodeDelayType(node.delay)"
                        size="small"
                        :bordered="false"
                        round
                        class="delay-tag"
                      >
                        {{ node.delay === 0 ? 'Не тестировалось' : node.delay + 'мс' }}
                      </n-tag>
                    </n-flex>

                    <n-flex align="center" class="node-type">
                      <n-tag size="small" :bordered="false" :type="getProxyTypeColor(node.type)">
                        {{ node.type }}
                      </n-tag>
                    </n-flex>

                    <n-flex justify="center" align="center">
                      <n-button
                        @click="testNodeDelay(node.name)"
                        :loading="testingNodes[node.name]"
                        secondary
                        size="small"
                        type="info"
                        ghost
                        class="proxy-button"
                      >
                        <template #icon>
                          <n-icon><speedometer-outline /></n-icon>
                        </template>
                        Тест скорости
                      </n-button>
                    </n-flex>
                  </n-space>
                </n-card>
              </n-grid-item>
            </n-grid>

            <n-empty
              v-if="filteredNodes.length === 0"
              description="Узлы не найдены"
              class="empty-container"
            />
          </n-tab-pane>
        </n-tabs>
      </n-card>
    </n-spin>
  </div>
</template>

<script lang="ts" setup>
import { onMounted, ref, computed, reactive, h, onUnmounted } from 'vue'
import { useMessage } from 'naive-ui'
import {
  RefreshOutline,
  CheckmarkCircleOutline,
  SwapHorizontalOutline,
  SpeedometerOutline,
  GlobeOutline,
  LayersOutline,
  HardwareChipOutline,
  SearchOutline,
  FlashOutline,
  ChevronDownOutline,
  InformationCircleOutline,
} from '@vicons/ionicons5'
import { useWindowSize } from '@vueuse/core'
import { Component } from 'vue'
import { tauriApi } from '@/services/tauri-api'
import { listen } from '@tauri-apps/api/event'

// Определение интерфейсов
interface ProxyHistory {
  time: string
  delay: number
}

interface ProxyData {
  type: string
  name: string
  now: string
  all: string[]
  history: ProxyHistory[]
  udp: boolean
}

interface Proxies {
  proxies: Record<string, ProxyData>
}

interface NodeInfo {
  name: string
  type: string
  delay: number
}

// Определение состояния
const message = useMessage()
const isLoading = ref(false)
const { width } = useWindowSize()

// Данные прокси
const rawProxies = ref<Record<string, ProxyData>>({})
const globalGroup = ref<ProxyData | null>(null)
const proxyGroups = ref<ProxyData[]>([])
const allNodes = ref<NodeInfo[]>([])
const testingNodes = reactive<Record<string, boolean>>({})
const currentProxyMode = ref('rule') // По умолчанию режим правил

// Переключение режима прокси
const isChangingMode = ref(false)
const showModeChangeModal = ref(false)
const targetProxyMode = ref('')

// Тестирование всех узлов
const batchTesting = ref(false)
const batchTestProgress = reactive({
  percentage: 0,
  text: 'Подготовка к тестированию...',
  current: 0,
  total: 0,
})

// Регистрация слушателей событий
let unlistenTestProgress: (() => void) | null = null
let unlistenTestResult: (() => void) | null = null
let unlistenTestComplete: (() => void) | null = null

// Опции режима прокси
const proxyModeOptions = [
  {
    label: 'Глобальный режим',
    key: 'global',
    icon: renderIcon(GlobeOutline),
  },
  {
    label: 'Режим правил',
    key: 'rule',
    icon: renderIcon(LayersOutline),
  },
]

// Вспомогательная функция для динамического рендеринга иконок
function renderIcon(icon: Component) {
  return () => h('div', { class: 'dropdown-option-icon' }, h(icon))
}

// Состояние вкладок
const activeTab = ref('global')
const activeGroupTab = ref('')
const searchText = ref('')

// Настройка количества колонок в сетке в зависимости от ширины окна
const gridCols = computed(() => {
  if (width.value < 640) return 1
  if (width.value < 960) return 2
  if (width.value < 1280) return 3
  return 4
})

// Фильтрация узлов
const filteredNodes = computed(() => {
  if (!searchText.value) return allNodes.value
  const keyword = searchText.value.toLowerCase()
  return allNodes.value.filter((node) => node.name.toLowerCase().includes(keyword))
})

// Жизненные циклы
onMounted(() => {
  init()
  // Получение текущего режима прокси
  getCurrentProxyMode()
  // Регистрация слушателей событий
  setupEventListeners()
})

onUnmounted(() => {
  // Очистка слушателей событий
  if (unlistenTestProgress) unlistenTestProgress()
  if (unlistenTestResult) unlistenTestResult()
  if (unlistenTestComplete) unlistenTestComplete()
})

// Настройка слушателей событий
const setupEventListeners = async () => {
  unlistenTestProgress = await listen('test-nodes-progress', (event) => {
    const data = event.payload as any
    batchTestProgress.current = data.current
    batchTestProgress.total = data.total
    batchTestProgress.percentage = (data.current / data.total) * 100
    batchTestProgress.text = `Тестирование: ${data.current}/${data.total} (${data.node})`
  })

  unlistenTestResult = await listen('test-node-result', (event) => {
    const data = event.payload as any
    // Обновление информации о задержке узла
    const nodeIndex = allNodes.value.findIndex((node) => node.name === data.name)
    if (nodeIndex !== -1) {
      allNodes.value[nodeIndex].delay = data.success ? data.delay : 0
    }
    // В случае неудачи можно отобразить сообщение об ошибке
    if (!data.success) {
      console.warn(`Тестирование узла ${data.name} не удалось: ${data.error}`)
    }
  })

  unlistenTestComplete = await listen('test-nodes-complete', () => {
    batchTesting.value = false
    message.success('Тестирование всех узлов завершено')
  })
}

/**
 * Инициализация и получение информации о прокси
 */
const init = async () => {
  isLoading.value = true
  try {
    // Использование Tauri API для получения информации о прокси
    const data = await tauriApi.proxy.getProxies()
    rawProxies.value = data.proxies

    // Извлечение глобальной группы
    if (data.proxies.GLOBAL) {
      globalGroup.value = data.proxies.GLOBAL
    }

    // Извлечение групп прокси
    const groups: ProxyData[] = []
    const nodes: NodeInfo[] = []

    Object.entries(data.proxies).forEach(([key, item]) => {
      // Исключение специальных групп и прямого подключения
      if (key === 'GLOBAL' || key === 'direct') return

      // Если это тип Selector или URLTest, добавляем в группы прокси
      if (item.type === 'Selector' || item.type === 'URLTest') {
        groups.push(item)

        // Если еще не установлен активный вкладка группы, устанавливаем первую найденную группу как активную
        if (!activeGroupTab.value && item.type === 'Selector') {
          activeGroupTab.value = item.name
        }
      }

      // Если это не тип группы, добавляем в список узлов
      if (!['Selector', 'URLTest', 'Fallback'].includes(item.type)) {
        const delay = item.history.length > 0 ? item.history[0].delay : 0
        nodes.push({
          name: item.name,
          type: item.type,
          delay,
        })
      }
    })

    proxyGroups.value = groups
    allNodes.value = nodes

    if (groups.length > 0 || nodes.length > 0) {
      message.success('Список прокси успешно загружен')
    }
  } catch (error) {
    console.error('Не удалось получить список прокси', error)
    message.error('Не удалось получить список прокси, проверьте, запущен ли Sing-Box')
  } finally {
    isLoading.value = false
  }
}

/**
 * Получение типа узла
 * @param name Имя узла
 * @returns Тип узла
 */
const getProxyType = (name: string): string => {
  if (rawProxies.value[name]) {
    return rawProxies.value[name].type
  }
  return 'Неизвестно'
}

/**
 * Получение цвета, соответствующего типу узла
 * @param type Тип узла
 * @returns Тип цвета
 */
const getProxyTypeColor = (type: string): string => {
  const typeMap: Record<string, string> = {
    Selector: 'info',
    URLTest: 'success',
    Fallback: 'warning',
    Direct: 'default',
    Hysteria2: 'error',
    Shadowsocks: 'warning',
    Trojan: 'primary',
    VMess: 'info',
    Socks5: 'default',
  }
  return typeMap[type] || 'default'
}

/**
 * Получение задержки узла
 * @param name Имя узла
 * @returns Значение задержки (миллисекунды)
 */
const getNodeDelay = (name: string): number => {
  if (rawProxies.value[name] && rawProxies.value[name].history.length > 0) {
    return rawProxies.value[name].history[0].delay
  }
  return 0
}

/**
 * Получение типа цвета, соответствующего задержке
 * @param delay Задержка (миллисекунды)
 * @returns Тип цвета
 */
const getNodeDelayType = (delay: number): string => {
  if (delay === 0) return 'default'
  if (delay < 100) return 'success'
  if (delay < 200) return 'info'
  if (delay < 300) return 'warning'
  return 'error'
}

/**
 * Получение текста, соответствующего режиму прокси
 * @param mode Режим прокси
 * @returns Текст режима
 */
const getProxyModeText = (mode: string): string => {
  const modeMap: Record<string, string> = {
    global: 'Глобальный режим',
    rule: 'Режим правил',
    tun: 'TUN режим',
  }
  return modeMap[mode] || 'Неизвестный режим'
}

/**
 * Проверка, является ли узел реальным (не группой)
 * @param name Имя узла
 * @returns Является ли узел реальным
 */
const isRealNode = (name: string): boolean => {
  if (!rawProxies.value[name]) return false
  return !['Selector', 'URLTest', 'Fallback'].includes(rawProxies.value[name].type)
}

/**
 * Тестирование задержки узла
 * @param name Имя узла
 * @param server URL тестового сервера
 */
const testNodeDelay = async (
  name: string,
  server: string = 'https://www.gstatic.com/generate_204',
) => {
  if (!rawProxies.value[name]) return

  // Установка состояния тестирования
  testingNodes[name] = true

  try {
    const data = await tauriApi.proxy.testNodeDelay(name, server)

    // Обновление информации о задержке узла
    const nodeIndex = allNodes.value.findIndex((node) => node.name === name)
    if (nodeIndex !== -1) {
      allNodes.value[nodeIndex].delay = data.delay
    }

    message.success(`Тестирование завершено: ${data.delay}мс`)
  } catch (error) {
    console.error('Не удалось провести тестирование', error)
    message.error('Не удалось провести тестирование, возможно узел не подключен или API не отвечает')
  } finally {
    // Очистка состояния тестирования
    testingNodes[name] = false
  }
}

/**
 * Тестирование всех узлов
 */
const batchTestAllNodes = async () => {
  if (batchTesting.value) return

  // Фильтрация узлов для тестирования
  const nodesToTest = allNodes.value
    .filter((node) => !['Direct', 'Reject'].includes(node.type))
    .map((node) => node.name)

  if (nodesToTest.length === 0) {
    message.warning('Нет узлов для тестирования')
    return
  }

  // Сброс прогресса
  batchTesting.value = true
  batchTestProgress.current = 0
  batchTestProgress.total = nodesToTest.length
  batchTestProgress.percentage = 0
  batchTestProgress.text = 'Подготовка к тестированию...'

  try {
    // Вызов API для проведения тестирования
    await tauriApi.proxy.batchTestNodes(nodesToTest)
  } catch (error) {
    console.error('Не удалось провести тестирование всех узлов', error)
    message.error('Не удалось провести тестирование всех узлов: ' + error)
    batchTesting.value = false
  }
}

/**
 * Переключение прокси
 * @param group Имя группы прокси
 * @param proxy Имя прокси для переключения
 */
const changeProxy = async (group: string, proxy: string) => {
  try {
    await tauriApi.proxy.changeProxy(group, proxy)
    message.success(`Переключено ${group} на ${proxy}`)
    // Перезагрузка данных
    await init()
  } catch (error) {
    console.error('Не удалось переключить', error)
    message.error('Не удалось переключить, проверьте, запущен ли Sing-Box')
  }
}

/**
 * Получение текущего режима прокси
 */
const getCurrentProxyMode = async () => {
  try {
    // Вызов API для получения текущего режима прокси
    const mode = await tauriApi.proxy.getCurrentProxyMode()
    currentProxyMode.value = mode
    console.log('Текущий режим прокси:', mode)
  } catch (error) {
    console.error('Не удалось получить режим прокси', error)
    // В случае ошибки используем режим правил по умолчанию
    currentProxyMode.value = 'rule'
  }
}

/**
 * Обработка изменения режима прокси
 */
const handleProxyModeChange = (key: string) => {
  if (key === currentProxyMode.value) return

  targetProxyMode.value = key
  showModeChangeModal.value = true
}

/**
 * Подтверждение переключения режима прокси
 */
const confirmProxyModeChange = async () => {
  if (!targetProxyMode.value) return

  isChangingMode.value = true
  try {
    // Вызов API для переключения режима прокси
    await tauriApi.proxy.toggleProxyMode(targetProxyMode.value)

    // Перезапуск ядра для применения изменений
    await tauriApi.kernel.restartKernel()

    // Обновление текущего режима
    currentProxyMode.value = targetProxyMode.value
    message.success(`Переключено на ${getProxyModeText(targetProxyMode.value)} и перезапущено ядро`)

    // Перезагрузка информации о прокси
    await init()
  } catch (error) {
    console.error('Не удалось переключить режим прокси', error)
    message.error(`Не удалось переключить режим прокси: ${error}`)
  } finally {
    isChangingMode.value = false
    showModeChangeModal.value = false
  }
}
</script>

<style scoped>
.proxy-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 16px 8px;
  animation: slide-up 0.4s ease;
}

.proxy-card {
  margin-bottom: 16px;
  border-radius: 16px;
  transition: all 0.3s ease;
  box-shadow: var(--shadow-light);
}

.proxy-card:hover,
.proxy-list-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-medium);
}

.proxy-list-card {
  border-radius: 16px;
  transition: all 0.3s ease;
  box-shadow: var(--shadow-light);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.card-title {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
  font-weight: 600;
}

.card-icon {
  color: var(--primary-color);
}

.refresh-button {
  transition: all 0.3s ease;
}

.refresh-button:hover:not(:disabled) {
  transform: translateY(-2px);
  color: var(--primary-color);
  background-color: rgba(64, 128, 255, 0.1);
}

.mode-tag {
  font-weight: 500;
  padding: 0 12px;
  height: 28px;
  display: flex;
  align-items: center;
  gap: 4px;
  cursor: pointer;
}

.mode-icon {
  margin-right: 4px;
}

.dropdown-icon {
  margin-left: 4px;
}

.dropdown-option-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  margin-right: 8px;
}

.modal-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
}

.modal-icon {
  color: var(--primary-color);
}

.modal-content {
  margin: 16px 0;
  line-height: 1.6;
}

.modal-footer {
  margin-top: 8px;
}

.proxy-group {
  margin-bottom: 20px;
}

.proxy-group-info {
  margin-bottom: 20px;
  padding: 0 4px;
}

.proxy-tag {
  font-weight: 500;
  padding: 0 12px;
  height: 28px;
}

.proxy-node-card {
  transition: all 0.3s ease;
  border-radius: 12px;
  border-left: 3px solid transparent;
}

.proxy-node-card:hover {
  transform: translateY(-3px);
  box-shadow: var(--shadow-medium);
}

.proxy-node-card-active {
  border-left: 3px solid var(--success-color);
}

.node-card {
  border-left: 3px solid var(--primary-color);
}

.proxy-name-container {
  font-weight: 500;
  flex: 1;
  overflow: hidden;
  color: var(--n-text-color-1);
}

.delay-tag,
.type-tag {
  font-weight: 500;
  transition: all 0.3s ease;
}

.node-type {
  margin-top: -8px;
}

.proxy-button {
  border-radius: 8px;
  font-weight: 500;
  transition: all 0.25s ease;
}

.proxy-button:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

:deep(.dark) .proxy-button:hover:not(:disabled) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.25);
}

.nodes-filter {
  margin-bottom: 20px;
}

.search-input {
  max-width: 320px;
  border-radius: 20px;
}

.empty-container {
  margin: 60px 0;
  opacity: 0.8;
}

:deep(.proxy-tabs .n-tabs-tab) {
  padding: 8px 16px;
  font-weight: 500;
  transition: all 0.3s ease;
}

:deep(.proxy-tabs .n-tabs-tab.n-tabs-tab--active) {
  font-weight: 600;
}

:deep(.proxy-tabs .n-tabs-tab-wrapper) {
  padding: 4px;
}

:deep(.n-tabs .n-tab-pane) {
  padding: 16px 0;
}

:deep(.n-card.proxy-node-card) {
  background-color: var(--card-color);
}

:deep(.n-card.proxy-node-card:hover) {
  background-color: var(--card-color-hover);
}

.batch-test-progress {
  margin-top: 16px;
  margin-bottom: 16px;
  width: 100%;
}

@keyframes slide-up {
  0% {
    opacity: 0;
    transform: translateY(20px);
  }
  100% {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
