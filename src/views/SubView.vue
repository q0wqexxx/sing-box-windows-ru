<template>
  <div class="sub-container">
    <!-- Карточка управления подписками -->
    <n-card class="sub-card" :bordered="false">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <n-h3 class="card-title">
              <n-icon size="24" class="card-icon">
                <link-outline />
              </n-icon>
              Управление подписками
            </n-h3>
            <n-tag :bordered="false" type="info" size="medium" class="sub-count-tag">
              {{ subStore.list.length }} подписок
            </n-tag>
          </div>
          <n-tooltip trigger="hover" placement="top">
            <template #trigger>
              <n-button
                quaternary
                circle
                size="medium"
                @click="showAddModal = true"
                :disabled="isLoading"
                class="add-button"
              >
                <template #icon>
                  <n-icon>
                    <add-outline />
                  </n-icon>
                </template>
              </n-button>
            </template>
            Добавить подписку
          </n-tooltip>
        </div>
      </template>

      <n-grid
        :x-gap="16"
        :y-gap="16"
        :cols="gridCols"
        responsive="screen"
        item-responsive
        :collapsed="false"
        :collapsed-rows="1"
      >
        <n-grid-item v-for="(item, index) in subStore.list" :key="index">
          <n-card
            :class="{
              'sub-node-card': true,
              'sub-node-card-active': subStore.activeIndex === index,
            }"
            :bordered="false"
            hoverable
          >
            <n-space vertical :size="14">
              <n-flex justify="space-between" align="center">
                <n-space align="center" :size="8" style="flex-wrap: nowrap; overflow: hidden">
                  <n-icon size="20" :color="subStore.activeIndex === index ? '#18a058' : '#4080ff'">
                    <link-outline />
                  </n-icon>
                  <n-text strong class="sub-name text-ellipsis">{{ item.name }}</n-text>
                  <div class="tag-container">
                    <n-tag
                      v-if="subStore.activeIndex === index"
                      type="success"
                      size="small"
                      :bordered="false"
                      class="active-tag"
                    >
                      Используется
                    </n-tag>
                    <n-tag
                      v-if="item.isManual"
                      type="warning"
                      size="small"
                      :bordered="false"
                      class="manual-tag"
                    >
                      Вручную
                    </n-tag>
                  </div>
                </n-space>
                <n-space :size="10">
                  <n-tooltip trigger="hover" placement="top">
                    <template #trigger>
                      <n-button
                        quaternary
                        circle
                        size="small"
                        @click="copyUrl(item.url)"
                        class="action-button"
                      >
                        <template #icon>
                          <n-icon>
                            <copy-outline />
                          </n-icon>
                        </template>
                      </n-button>
                    </template>
                    Копировать ссылку
                  </n-tooltip>

                  <n-tooltip trigger="hover" placement="top">
                    <template #trigger>
                      <n-button
                        quaternary
                        circle
                        size="small"
                        @click="handleEdit(index, item)"
                        class="action-button"
                      >
                        <template #icon>
                          <n-icon>
                            <create-outline />
                          </n-icon>
                        </template>
                      </n-button>
                    </template>
                    Редактировать подписку
                  </n-tooltip>

                  <!-- Новое: кнопка просмотра/редактирования текущей конфигурации -->
                  <n-tooltip v-if="subStore.activeIndex === index" trigger="hover" placement="top">
                    <template #trigger>
                      <n-button
                        quaternary
                        circle
                        size="small"
                        type="info"
                        @click="editCurrentConfig()"
                        class="action-button"
                      >
                        <template #icon>
                          <n-icon>
                            <code-outline />
                          </n-icon>
                        </template>
                      </n-button>
                    </template>
                    Редактировать текущую конфигурацию
                  </n-tooltip>

                  <n-popconfirm
                    @positive-click="deleteSubscription(index)"
                    positive-text="Удалить"
                    negative-text="Отмена"
                  >
                    <template #trigger>
                      <n-button
                        quaternary
                        circle
                        size="small"
                        type="error"
                        :disabled="subStore.activeIndex === index"
                        class="action-button"
                      >
                        <template #icon>
                          <n-icon>
                            <trash-outline />
                          </n-icon>
                        </template>
                      </n-button>
                    </template>
                    Вы уверены, что хотите удалить эту подписку?
                  </n-popconfirm>
                </n-space>
              </n-flex>

              <div class="url-container">
                <n-ellipsis style="max-width: 100%" :tooltip="{ width: 'trigger' }">
                  {{ item.url }}
                </n-ellipsis>
              </div>

              <n-flex justify="space-between" align="center">
                <n-text depth="3" class="update-time">
                  {{ item.lastUpdate ? formatTime(item.lastUpdate) : 'Никогда не использовалось' }}
                </n-text>
                <n-button
                  secondary
                  size="small"
                  :loading="item.isLoading"
                  @click="useSubscription(item.url, index)"
                  :type="subStore.activeIndex === index ? 'success' : 'primary'"
                  :ghost="subStore.activeIndex !== index"
                  class="use-button"
                >
                  <template #icon>
                    <n-icon>
                      <checkmark-circle-outline v-if="subStore.activeIndex === index" />
                      <play-circle-outline v-else />
                    </n-icon>
                  </template>
                  {{ subStore.activeIndex === index ? 'Использовать снова' : 'Использовать' }}
                </n-button>
              </n-flex>
            </n-space>
          </n-card>
        </n-grid-item>
      </n-grid>

      <n-empty v-if="!subStore.list.length" description="Нет подписок" class="empty-container">
        <template #extra>
          <n-button type="primary" @click="showAddModal = true" class="add-sub-button">
            <template #icon>
              <n-icon><add-outline /></n-icon>
            </template>
            Добавить подписку
          </n-button>
        </template>
      </n-empty>
    </n-card>
  </div>

  <!-- Диалог добавления/редактирования подписки -->
  <n-modal
    v-model:show="showAddModal"
    :mask-closable="false"
    preset="dialog"
    :title="editIndex === null ? 'Добавить подписку' : 'Редактировать подписку'"
    :bordered="false"
    style="width: 600px"
    class="sub-modal"
  >
    <n-form
      ref="formRef"
      :model="formValue"
      :rules="rules"
      label-placement="left"
      label-width="80"
      require-mark-placement="right-hanging"
    >
      <n-form-item label="Название" path="name">
        <n-input
          v-model:value="formValue.name"
          placeholder="Введите название подписки"
          @keydown.enter.prevent
          class="form-input"
        />
      </n-form-item>

      <n-tabs type="line" animated v-model:value="activeTab" class="sub-tabs">
        <n-tab-pane name="url" tab="Добавить по URL">
          <n-form-item label="Ссылка" path="url">
            <n-input
              v-model:value="formValue.url"
              type="textarea"
              placeholder="Введите ссылку на подписку"
              :autosize="{ minRows: 2, maxRows: 4 }"
              class="form-input"
            />
          </n-form-item>
        </n-tab-pane>
        <n-tab-pane name="manual" tab="Редактировать вручную">
          <n-form-item label="Содержание" path="manualContent">
            <n-input
              v-model:value="formValue.manualContent"
              type="textarea"
              placeholder="Введите содержание конфигурации (в формате JSON)"
              :autosize="{ minRows: 8, maxRows: 20 }"
              class="form-input code-input"
            />
          </n-form-item>
        </n-tab-pane>
      </n-tabs>
    </n-form>
    <template #action>
      <n-space justify="end">
        <n-button @click="handleCancel" class="modal-button">Отмена</n-button>
        <n-button type="primary" @click="handleConfirm" :loading="isLoading" class="modal-button">
          Подтвердить
        </n-button>
      </n-space>
    </template>
  </n-modal>

  <!-- Диалог редактирования текущей конфигурации -->
  <n-modal
    v-model:show="showConfigModal"
    :mask-closable="false"
    preset="dialog"
    title="Редактировать текущую конфигурацию"
    :bordered="false"
    style="width: 800px"
    class="config-modal"
  >
    <n-input
      v-model:value="currentConfig"
      type="textarea"
      placeholder="Содержание конфигурации (в формате JSON)"
      :autosize="{ minRows: 15, maxRows: 30 }"
      class="form-input code-input"
    />
    <template #action>
      <n-space justify="end">
        <n-button @click="showConfigModal = false" class="modal-button">Отмена</n-button>
        <n-button
          type="primary"
          @click="saveCurrentConfig"
          :loading="isConfigLoading"
          class="modal-button"
        >
          Сохранить и применить
        </n-button>
      </n-space>
    </template>
  </n-modal>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { useMessage } from 'naive-ui'
import { ref, computed } from 'vue'
import { useSubStore } from '@/stores/SubStore'
import {
  AddOutline,
  LinkOutline,
  CopyOutline,
  CreateOutline,
  TrashOutline,
  CheckmarkCircleOutline,
  PlayCircleOutline,
  CodeOutline,
} from '@vicons/ionicons5'
import type { FormInst, FormRules } from 'naive-ui'
import { useWindowSize } from '@vueuse/core'

interface Subscription {
  name: string
  url: string
  lastUpdate?: number
  isLoading: boolean
  isManual: boolean
  manualContent?: string
}

const message = useMessage()
const subStore = useSubStore()
const showAddModal = ref(false)
const editIndex = ref<number | null>(null)
const formRef = ref<FormInst | null>(null)
const isLoading = ref(false)
const { width } = useWindowSize()
const activeTab = ref('url')

// Переменные для редактирования текущей конфигурации
const showConfigModal = ref(false)
const currentConfig = ref('')
const isConfigLoading = ref(false)

// Настройка количества колонок в сетке в зависимости от ширины окна
const gridCols = computed(() => {
  if (width.value < 768) return 1
  if (width.value < 1200) return 2
  return 3
})

const formValue = ref<Subscription>({
  name: '',
  url: '',
  isLoading: false,
  isManual: false,
  manualContent: '',
})

const rules: FormRules = {
  name: [{ required: true, message: 'Введите название подписки', trigger: 'blur' }],
  url: [
    {
      required: true,
      message: 'Введите ссылку на подписку',
      trigger: 'blur',
      validator: (rule, value) => {
        // Проверка URL только в режиме URL
        return activeTab.value === 'url' ? !!value : true
      },
    },
    {
      type: 'url',
      message: 'Введите действительный URL',
      trigger: 'blur',
      validator: (rule, value) => {
        // Проверка формата URL только в режиме URL
        return activeTab.value === 'url' ? true : true
      },
    },
  ],
  manualContent: [
    {
      required: true,
      message: 'Введите содержание конфигурации',
      trigger: 'blur',
      validator: (rule, value) => {
        // Проверка содержания только в режиме ручного редактирования
        return activeTab.value === 'manual' ? !!value : true
      },
    },
  ],
}

const resetForm = () => {
  formValue.value = {
    name: '',
    url: '',
    isLoading: false,
    isManual: false,
    manualContent: '',
  }
  editIndex.value = null
}

const handleEdit = (index: number, item: Subscription) => {
  editIndex.value = index
  formValue.value = {
    name: item.name,
    url: item.url,
    isLoading: item.isLoading,
    isManual: item.isManual,
    manualContent: item.manualContent,
  }
  // Установка активной вкладки в зависимости от типа подписки
  activeTab.value = item.isManual ? 'manual' : 'url'
  showAddModal.value = true
}

const handleConfirm = () => {
  formRef.value?.validate(async (errors) => {
    if (!errors) {
      try {
        isLoading.value = true

        // Определение режима редактирования
        const isManual = activeTab.value === 'manual'

        if (isManual && formValue.value.manualContent) {
          // Если режим ручного редактирования и есть содержание, сохраняем его
          if (editIndex.value === null) {
            // Если это новая подписка, используем это содержание
            await invoke('add_manual_subscription', { content: formValue.value.manualContent })
          }
        } else if (!isManual) {
          // Если режим URL и это новая подписка
          if (editIndex.value === null) {
            await invoke('download_subscription', { url: formValue.value.url })
          }
        }

        if (editIndex.value === null) {
          // Добавление новой подписки
          subStore.list.push({
            name: formValue.value.name,
            url: formValue.value.url,
            lastUpdate: isManual ? Date.now() : undefined,
            isLoading: false,
            isManual: isManual,
            manualContent: isManual ? formValue.value.manualContent : undefined,
          })

          // Если это новая ручная конфигурация, автоматически устанавливаем ее активной
          if (isManual) {
            subStore.activeIndex = subStore.list.length - 1
          }

          message.success('Подписка успешно добавлена')
        } else {
          // Обновление подписки
          subStore.list[editIndex.value] = {
            ...subStore.list[editIndex.value],
            name: formValue.value.name,
            url: formValue.value.url,
            isManual: isManual,
            manualContent: isManual ? formValue.value.manualContent : undefined,
          }
          message.success('Подписка успешно обновлена')
        }
        showAddModal.value = false
        resetForm()
      } catch (error) {
        message.error('Ошибка операции: ' + error)
      } finally {
        isLoading.value = false
      }
    }
  })
}

const handleCancel = () => {
  showAddModal.value = false
  resetForm()
}

const deleteSubscription = (index: number) => {
  if (subStore.activeIndex === index) {
    message.warning('Нельзя удалить текущую активную подписку')
    return
  }
  subStore.list.splice(index, 1)
  message.success('Подписка удалена')
}

const useSubscription = async (url: string, index: number) => {
  try {
    // Установка состояния загрузки
    subStore.list[index].isLoading = true

    const item = subStore.list[index]

    if (item.isManual && item.manualContent) {
      // Если это ручная конфигурация, используем сохраненное содержание
      await invoke('add_manual_subscription', { content: item.manualContent })
    } else {
      // Иначе загружаем содержание по URL
      await invoke('download_subscription', { url })
    }

    // Обновление состояния подписки
    subStore.list[index].lastUpdate = Date.now()
    subStore.activeIndex = index
    message.success('Подписка успешно использована')
  } catch (error) {
    message.error('Ошибка использования подписки: ' + error)
  } finally {
    subStore.list[index].isLoading = false
  }
}

const copyUrl = (url: string) => {
  navigator.clipboard.writeText(url)
  message.success('Ссылка скопирована в буфер обмена')
}

const formatTime = (timestamp: number): string => {
  const date = new Date(timestamp)
  return `Последнее обновление: ${date.toLocaleString('ru-RU', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })}`
}

const editCurrentConfig = async () => {
  try {
    isConfigLoading.value = true
    // Получение текущего содержания конфигурации
    const config = await invoke('get_current_config')
    if (typeof config === 'string') {
      currentConfig.value = config
      showConfigModal.value = true
    }
  } catch (error) {
    message.error('Ошибка чтения конфигурации: ' + error)
  } finally {
    isConfigLoading.value = false
  }
}

const saveCurrentConfig = async () => {
  try {
    isConfigLoading.value = true

    // Сохранение содержания конфигурации
    await invoke('add_manual_subscription', {
      content: currentConfig.value,
    })

    // Если текущая активная подписка является ручной конфигурацией, обновляем ее содержание
    if (subStore.activeIndex !== null) {
      const activeItem = subStore.list[subStore.activeIndex]
      if (activeItem.isManual) {
        subStore.list[subStore.activeIndex].manualContent = currentConfig.value
        subStore.list[subStore.activeIndex].lastUpdate = Date.now()
      }
    }

    message.success('Конфигурация сохранена и применена')
    showConfigModal.value = false
  } catch (error) {
    message.error('Ошибка сохранения конфигурации: ' + error)
  } finally {
    isConfigLoading.value = false
  }
}
</script>

<style scoped>
.sub-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 16px 8px;
  animation: slide-up 0.4s ease;
}

.sub-card {
  border-radius: 16px;
  transition: all 0.3s ease;
  box-shadow: var(--shadow-light);
}

.sub-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-medium);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
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

.sub-count-tag {
  font-weight: 500;
  padding: 0 12px;
  height: 28px;
  background-color: rgba(144, 147, 153, 0.12);
  color: var(--n-text-color-2);
}

.add-button {
  transition: all 0.3s ease;
}

.add-button:hover:not(:disabled) {
  transform: translateY(-2px);
  color: var(--primary-color);
  background-color: rgba(64, 128, 255, 0.1);
}

.sub-node-card {
  border-radius: 12px;
  transition: all 0.3s ease;
  border-left: 3px solid transparent;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.sub-node-card:hover {
  transform: translateY(-3px);
  box-shadow: var(--shadow-medium);
}

.sub-node-card-active {
  border-left: 3px solid var(--success-color);
  background-color: rgba(0, 180, 42, 0.05);
}

.sub-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--n-text-color-1);
  max-width: 120px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.text-ellipsis {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tag-container {
  display: flex;
  flex-wrap: nowrap;
  gap: 4px;
  min-width: 0;
}

.active-tag {
  font-weight: 500;
  padding: 2px 8px;
  white-space: nowrap;
}

.manual-tag {
  font-weight: 500;
  padding: 2px 8px;
  white-space: nowrap;
}

.url-container {
  padding: 8px 10px;
  border-radius: 8px;
  font-family: monospace;
  font-size: 13px;
  color: var(--n-text-color-2);
  word-break: break-all;
  border: 1px solid var(--n-border-color);
  flex-grow: 1;
  margin: 8px 0;
  display: flex;
  align-items: center;
}

.update-time {
  font-size: 12px;
  color: var(--n-text-color-3);
}

.use-button {
  border-radius: 8px;
  font-weight: 500;
  transition: all 0.25s ease;
}

.use-button:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

:deep(.dark) .use-button:hover:not(:disabled) {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.25);
}

.action-button {
  transition: all 0.3s ease;
}

.action-button:hover {
  transform: translateY(-1px);
}

.empty-container {
  margin: 60px 0;
  opacity: 0.8;
}

.add-sub-button {
  font-weight: 500;
  border-radius: 8px;
  padding: 0 20px;
  height: 36px;
  transition: all 0.3s ease;
}

.add-sub-button:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-medium);
}

.sub-modal {
  border-radius: 16px;
}

.form-input {
  transition: all 0.3s ease;
}

.form-input:hover {
  box-shadow: var(--shadow-focus);
}

.modal-button {
  min-width: 80px;
  border-radius: 8px;
  font-weight: 500;
}

/* Новое: стили */
.sub-tabs {
  margin-top: 10px;
}

.code-input {
  font-family: monospace;
  font-size: 13px;
  background-color: rgba(0, 0, 0, 0.02);
}

:deep(.dark) .code-input {
  background-color: rgba(255, 255, 255, 0.05);
}

.manual-icon {
  margin-right: 4px;
  color: #ff9800;
}

.sub-node-card > :deep(.n-card__content) {
  display: flex;
  flex-direction: column;
  flex-grow: 1;
}

.sub-node-card > :deep(.n-card__content) > .n-space {
  height: 100%;
  display: flex;
  flex-direction: column;
}
</style>
