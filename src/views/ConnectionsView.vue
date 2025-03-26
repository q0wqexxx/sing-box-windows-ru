<template>
  <div class="connections-container">
    <n-card class="connections-card" :bordered="false">
      <template #header>
        <div class="card-header">
          <h2>Список соединений</h2>
          <n-space>
            <n-button type="primary" @click="refreshConnections" :loading="loading">
              <template #icon>
                <n-icon><refresh-outline /></n-icon>
              </template>
              Обновить
            </n-button>
          </n-space>
        </div>
      </template>

      <n-spin :show="loading">
        <div class="stats-bar">
          <n-space justify="space-between" align="center">
            <n-statistic label="Активные соединения">
              {{ connections.length }}
            </n-statistic>
            <n-space>
              <n-statistic label="Общий объем загрузки">
                {{ formatBytes(connectionsTotal.upload) }}
              </n-statistic>
              <n-statistic label="Общий объем скачивания">
                {{ formatBytes(connectionsTotal.download) }}
              </n-statistic>
            </n-space>
          </n-space>
        </div>

        <div v-if="connections.length > 0" class="connections-list">
          <n-data-table
            :columns="columns"
            :data="connections"
            :pagination="pagination"
            :bordered="false"
            :max-height="600"
            striped
          />
        </div>
        <n-empty v-else description="Нет активных соединений" />
      </n-spin>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, h, computed } from 'vue'
import { useMessage, NTag, DataTableColumns, NSpace, NTooltip, NText } from 'naive-ui'
import { RefreshOutline } from '@vicons/ionicons5'
import { useInfoStore } from '@/stores/infoStore'

const message = useMessage()
const loading = ref(false)
const infoStore = useInfoStore()

// Использование вычисляемых свойств для получения информации о соединениях
const connections = computed(() => infoStore.connections)
const connectionsTotal = computed(() => infoStore.connectionsTotal)

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

// Форматирование размера в байтах
const formatBytes = (bytes: number) => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return (bytes / Math.pow(k, i)).toFixed(2) + ' ' + sizes[i]
}

// Форматирование времени
const formatTime = (timeString: string) => {
  try {
    const date = new Date(timeString)
    return date.toLocaleString()
  } catch (e) {
    return timeString
  }
}

// Определение столбцов таблицы
const columns: DataTableColumns<Connection> = [
  {
    title: 'ID',
    key: 'id',
    width: 100,
    ellipsis: {
      tooltip: true,
    },
  },
  {
    title: 'Время начала',
    key: 'start',
    width: 160,
    render(row: Connection) {
      return formatTime(row.start)
    },
  },
  {
    title: 'Сеть/Тип',
    key: 'network',
    width: 120,
    render(row: Connection) {
      const { network, type } = row.metadata
      return h(
        NSpace,
        { vertical: true, size: 'small' },
        {
          default: () => [
            h(
              NTag,
              {
                type: network === 'tcp' ? 'info' : 'warning',
                size: 'small',
                bordered: false,
              },
              { default: () => network.toUpperCase() },
            ),
            h(
              NTag,
              {
                type: 'default',
                size: 'small',
                bordered: false,
              },
              { default: () => type },
            ),
          ],
        },
      )
    },
  },
  {
    title: 'Источник',
    key: 'source',
    width: 200,
    render(row: Connection) {
      const { sourceIP, sourcePort } = row.metadata
      return h(
        NTooltip,
        {},
        {
          trigger: () => `${sourceIP}:${sourcePort}`,
          default: () =>
            h('div', {}, [h('div', {}, `IP: ${sourceIP}`), h('div', {}, `Порт: ${sourcePort}`)]),
        },
      )
    },
  },
  {
    title: 'Назначение',
    key: 'destination',
    width: 200,
    render(row: Connection) {
      const { destinationIP, destinationPort, host } = row.metadata
      return h(
        NTooltip,
        {},
        {
          trigger: () => host || `${destinationIP}:${destinationPort}`,
          default: () =>
            h('div', {}, [
              host ? h('div', {}, `Хост: ${host}`) : null,
              h('div', {}, `IP: ${destinationIP}`),
              h('div', {}, `Порт: ${destinationPort}`),
            ]),
        },
      )
    },
  },
  {
    title: 'Правило',
    key: 'rule',
    width: 160,
    render(row: Connection) {
      return h(
        NSpace,
        { vertical: true, size: 'small' },
        {
          default: () => [
            h(
              NTag,
              {
                type: 'success',
                size: 'small',
                bordered: false,
              },
              { default: () => row.rule },
            ),
            row.rulePayload
              ? h(NText, { depth: 3, size: 'small' }, { default: () => row.rulePayload })
              : null,
          ],
        },
      )
    },
  },
  {
    title: 'Процесс',
    key: 'process',
    ellipsis: {
      tooltip: true,
    },
    render(row: Connection) {
      return row.metadata.processPath || 'Неизвестно'
    },
  },
  {
    title: 'Трафик',
    key: 'traffic',
    width: 160,
    render(row: Connection) {
      return h(
        NSpace,
        { vertical: true, size: 'small' },
        {
          default: () => [
            h(
              NSpace,
              { align: 'center', size: 'small' },
              {
                default: () => [
                  h(
                    NTag,
                    { type: 'error', size: 'small', bordered: false },
                    { default: () => '↑' },
                  ),
                  h(NText, {}, { default: () => formatBytes(row.upload) }),
                ],
              },
            ),
            h(
              NSpace,
              { align: 'center', size: 'small' },
              {
                default: () => [
                  h(NTag, { type: 'info', size: 'small', bordered: false }, { default: () => '↓' }),
                  h(NText, {}, { default: () => formatBytes(row.download) }),
                ],
              },
            ),
          ],
        },
      )
    },
  },
]

// Настройки пагинации
const pagination = {
  pageSize: 10,
}

// Обновление списка соединений
const refreshConnections = async () => {
  loading.value = true
  try {
    // Здесь фактически ничего не нужно делать, так как соединения в infoStore автоматически обновляются через WebSocket
    // Но мы все равно предоставляем кнопку обновления для удобства пользователя
    message.success('Список соединений обновлен')
  } catch (error) {
    console.error('Не удалось обновить список соединений:', error)
    message.error(`Не удалось обновить список соединений: ${error}`)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  // При монтировании компонента, убедитесь, что данные соединений в infoStore уже инициализированы
  if (!connections.value.length && infoStore.uptime > 0) {
    refreshConnections()
  }
})
</script>

<style scoped>
.connections-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 12px 8px;
}

.connections-card {
  border-radius: 16px;
  transition: all 0.3s ease;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.05);
}

.connections-card :deep(.n-card__content) {
  padding: 16px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-header h2 {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 500;
}

.stats-bar {
  margin-bottom: 16px;
  padding: 12px;
  background-color: var(--n-color-container);
  border-radius: 8px;
}

.connections-list {
  margin-top: 12px;
}
</style>
