<template>
  <div class="setting-container">
    <!-- Карточка управления ядром -->
    <n-card class="setting-card" :bordered="false">
      <template #header-extra>
        <n-space align="center" :size="12">
          <n-tag
            v-if="infoStore.version.version"
            :bordered="false"
            type="default"
            size="medium"
            class="version-tag"
          >
            Текущая версия: {{ infoStore.version.version }}
          </n-tag>
          <n-tag v-else :bordered="false" type="error" size="medium" class="version-tag">
            Ядро не установлено
          </n-tag>
          <n-tag
            v-if="hasNewVersion"
            :bordered="false"
            type="warning"
            size="medium"
            class="version-tag"
          >
            Новая версия: {{ infoStore.newVersion }}
          </n-tag>
        </n-space>
      </template>
      <template #header>
        <div class="card-header">
          <n-h3 class="card-title">
            <n-icon size="24" class="card-icon">
              <settings-outline />
            </n-icon>
            Управление ядром
          </n-h3>
        </div>
      </template>

      <n-space vertical :size="20">
        <n-alert
          v-if="hasNewVersion"
          type="warning"
          :show-icon="true"
          title="Обнаружена новая версия"
          class="version-alert"
        >
          Доступна новая версия ядра. Рекомендуется обновить для улучшения работы.
        </n-alert>

        <n-alert
          v-if="!infoStore.version.version"
          type="error"
          :show-icon="true"
          title="Ядро не установлено"
          class="version-alert"
        >
          Пожалуйста, скачайте и установите ядро перед использованием.
        </n-alert>

        <n-progress
          v-if="downloading"
          type="line"
          :percentage="downloadProgress"
          :processing="downloadProgress < 100"
          :indicator-placement="'inside'"
          :rail-style="{ background: 'var(--n-color-disabled)' }"
          class="download-progress"
        >
          {{ downloadMessage }}
        </n-progress>

        <n-space align="center" justify="space-between">
          <n-button
            type="primary"
            @click="downloadTheKernel"
            :loading="loading"
            :disabled="downloading"
            size="medium"
            class="download-button"
          >
            <template #icon>
              <n-icon>
                <download-outline />
              </n-icon>
            </template>
            {{
              hasNewVersion
                ? 'Скачать новую версию'
                : infoStore.version.version
                  ? 'Скачать текущую версию'
                  : 'Скачать ядро'
            }}
          </n-button>

          <n-space :size="16">
            <n-button
              text
              size="medium"
              @click="showManualDownloadModal"
              :disabled="downloading"
              class="action-button"
            >
              Скачивание вручную
            </n-button>
            <n-button text size="medium" @click="checkManualInstall" :disabled="downloading">
              Проверить установку
            </n-button>
          </n-space>
        </n-space>

        <n-alert v-if="downloadError" type="error" :show-icon="true" style="margin-top: 16px">
          <template #header> Скачивание не удалось </template>
          <div style="white-space: pre-line">{{ downloadError }}</div>
        </n-alert>
      </n-space>
    </n-card>

    <!-- Карточка настроек запуска -->
    <n-card class="setting-card" :bordered="false">
      <template #header>
        <div class="card-header">
          <n-h3 class="card-title">
            <n-icon size="20" class="card-icon">
              <power-outline />
            </n-icon>
            Настройки запуска
          </n-h3>
        </div>
      </template>

      <n-list>
        <n-list-item>
          <n-space justify="space-between" align="center" style="width: 100%">
            <div class="setting-item">
              <div class="setting-title">Автозапуск при включении</div>
              <div class="setting-desc">
                {{ appStore.autoStartApp ? 'Приложение будет автоматически запускаться при включении системы' : 'Приложение нужно запускать вручную' }}
              </div>
            </div>
            <n-switch v-model:value="appStore.autoStartApp" @update-value="onAutoStartChange">
              <template #checked>Вкл</template>
              <template #unchecked>Выкл</template>
            </n-switch>
          </n-space>
        </n-list-item>

        <n-list-item>
          <n-space justify="space-between" align="center" style="width: 100%">
            <div class="setting-item">
              <div class="setting-title">Автозапуск ядра</div>
              <div class="setting-desc">
                {{ appStore.autoStartKernel ? 'Ядро будет автоматически запускаться при запуске приложения' : 'Ядро нужно запускать вручную' }}
              </div>
            </div>
            <n-switch v-model:value="appStore.autoStartKernel">
              <template #checked>Вкл</template>
              <template #unchecked>Выкл</template>
            </n-switch>
          </n-space>
        </n-list-item>

        <n-list-item>
          <n-space justify="space-between" align="center" style="width: 100%">
            <div class="setting-item">
              <div class="setting-title">Предпочитать IPv6</div>
              <div class="setting-desc">
                {{ appStore.preferIpv6 ? 'Предпочитать использование IPv6 соединений' : 'Использовать только IPv4 соединения' }}
              </div>
            </div>
            <n-switch v-model:value="appStore.preferIpv6" @update-value="onIpVersionChange">
              <template #checked>Вкл</template>
              <template #unchecked>Выкл</template>
            </n-switch>
          </n-space>
        </n-list-item>
      </n-list>
    </n-card>

    <!-- Карточка "О программе" -->
    <n-card class="setting-card" :bordered="false">
      <template #header>
        <div class="card-header">
          <n-h3 class="card-title">
            <n-icon size="20" class="card-icon">
              <information-circle-outline />
            </n-icon>
            О программе
          </n-h3>
        </div>
      </template>

      <n-grid :cols="2" :x-gap="12" :y-gap="8">
        <n-gi>
          <div class="about-item">
            <span class="about-label">Версия приложения</span>
            <n-space align="center">
              <span class="about-value">{{ appStore.appVersion }}</span>
              <n-button text size="tiny" @click="handleCheckUpdate" :loading="checkingUpdate">
                <template #icon>
                  <n-icon><refresh-outline /></n-icon>
                </template>
                Проверить обновления
              </n-button>
            </n-space>
          </div>
        </n-gi>
        <n-gi>
          <div class="about-item">
            <span class="about-label">Версия ядра</span>
            <span class="about-value">{{ infoStore.version.version }}</span>
          </div>
        </n-gi>
        <n-gi>
          <div class="about-item">
            <span class="about-label">Система</span>
            <span class="about-value">Windows</span>
          </div>
        </n-gi>
        <n-gi>
          <div class="about-item">
            <span class="about-label">Лицензия</span>
            <span class="about-value">MIT License</span>
          </div>
        </n-gi>
      </n-grid>

      <div class="about-footer">
        <n-space justify="center" align="center">
          <n-button
            text
            tag="a"
            href="https://github.com/xinggaoya/sing-box-windows"
            target="_blank"
          >
            <template #icon>
              <n-icon><logo-github /></n-icon>
            </template>
            GitHub
          </n-button>
          <n-divider vertical />
          <n-button
            text
            tag="a"
            href="https://github.com/xinggaoya/sing-box-windows"
            target="_blank"
          >
            <template #icon>
              <n-icon><globe-outline /></n-icon>
            </template>
            Веб-сайт
          </n-button>
        </n-space>
      </div>
    </n-card>
  </div>

  <!-- Диалог обновления приложения -->
  <update-modal
    v-model:show="showUpdateModal"
    :latest-version="latestVersion"
    :current-version="appStore.appVersion"
    :download-url="downloadUrl"
    @update="handleUpdate"
    @cancel="skipUpdate"
  />
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useMessage, useDialog } from 'naive-ui'
import { enable, disable } from '@tauri-apps/plugin-autostart'
import { useInfoStore } from '@/stores/infoStore'
import { useAppStore } from '@/stores/AppStore'
import {
  DownloadOutline,
  SettingsOutline,
  PowerOutline,
  InformationCircleOutline,
  LogoGithub,
  GlobeOutline,
  RefreshOutline,
} from '@vicons/ionicons5'
import { listen } from '@tauri-apps/api/event'
import { tauriApi } from '@/services/tauri-api'
import { appDataDir } from '@tauri-apps/api/path'
import UpdateModal from '@/components/UpdateModal.vue'

const message = useMessage()
const dialog = useDialog()
const appStore = useAppStore()
const infoStore = useInfoStore()
const loading = ref(false)
const downloading = ref(false)
const downloadProgress = ref(0)
const downloadMessage = ref('')

// Состояние обновления
const showUpdateModal = ref(false)
const latestVersion = ref('')
const downloadUrl = ref('')
const skipUpdateFlag = ref(false)

// Проверка состояния обновления
const checkingUpdate = ref(false)

// Новое состояние
const downloadError = ref<string | null>(null)
const appDataPath = ref('')

// Проверка обновлений
const checkUpdate = async () => {
  try {
    if (skipUpdateFlag.value) return

    const result = await tauriApi.update.checkUpdate(appStore.appVersion)
    if (result.has_update) {
      showUpdateModal.value = true
      latestVersion.value = result.latest_version
      downloadUrl.value = result.download_url
    }
  } catch (error) {
    console.error('Проверка обновлений не удалась:', error)
  }
}

// Обработка обновления
const handleUpdate = async () => {
  try {
    await tauriApi.update.downloadAndInstallUpdate(downloadUrl.value)
  } catch (error) {
    message.error('Обновление не удалось: ' + error)
  }
}

// Пропуск обновления
const skipUpdate = () => {
  showUpdateModal.value = false
  skipUpdateFlag.value = true
}

// Ручная проверка обновлений
const handleCheckUpdate = async () => {
  try {
    checkingUpdate.value = true
    const result = await appStore.checkUpdate(false)
    if (result?.has_update) {
      showUpdateModal.value = true
      latestVersion.value = result.latest_version
      downloadUrl.value = result.download_url
      message.success(`Обнаружена новая версия: ${result.latest_version}`)
    } else {
      message.info('Установлена последняя версия')
    }
  } catch (error) {
    message.error(`Проверка обновлений не удалась: ${error}`)
  } finally {
    checkingUpdate.value = false
  }
}

const hasNewVersion = computed(() => {
  if (!infoStore.newVersion || !infoStore.version.version) return false
  return infoStore.newVersion != infoStore.version.version
})

const downloadTheKernel = async () => {
  try {
    loading.value = true
    downloading.value = true
    downloadProgress.value = 0
    downloadMessage.value = 'Подготовка к скачиванию...'
    downloadError.value = null

    await tauriApi.subscription.downloadLatestKernel()

    // Обновление информации о версии после успешного скачивания
    await infoStore.updateVersion()
  } catch (error) {
    downloadError.value = error as string
    message.error(error as string)
  } finally {
    downloading.value = false
    loading.value = false
  }
}

// Настройка автозапуска при включении
const onAutoStartChange = async (value: boolean) => {
  try {
    if (value) {
      await enable()
      message.success('Автозапуск при включении включен')
    } else {
      await disable()
      message.success('Автозапуск при включении отключен')
    }
  } catch (error) {
    message.error(`Настройка не удалась: ${error}`)
    // Восстановление предыдущей настройки
    appStore.autoStartApp = !value
  }
}

const onIpVersionChange = async (value: boolean) => {
  try {
    await tauriApi.proxy.toggleIpVersion(value)
    // Перезапуск ядра после переключения
    if (appStore.isRunning) {
      await tauriApi.kernel.restartKernel()
    }
  } catch (error: unknown) {
    message.error(`Настройка не удалась: ${error instanceof Error ? error.message : String(error)}`)
    // Откат состояния
    appStore.preferIpv6 = !value
  }
}

// Показать инструкцию по ручному скачиванию
const showManualDownloadModal = () => {
  dialog.info({
    title: 'Инструкция по ручному скачиванию',
    content: `Пожалуйста, выполните следующие шаги:
1. Перейдите по ссылке https://github.com/SagerNet/sing-box/releases/latest
2. Скачайте версию sing-box для вашей системы
3. Поместите распакованный файл sing-box.exe в следующую директорию:
Пользовательская директория/AppData/Local/sing-box-windows/sing-box/

После завершения нажмите кнопку "Проверить установку" для проверки успешности установки.`,
    positiveText: 'Понятно',
  })
}

// Проверка ручной установки
const checkManualInstall = async () => {
  try {
    loading.value = true
    const success = await infoStore.checkKernelVersion()
    if (success) {
      message.success('Проверка установки ядра успешна!')
    } else {
      message.error('Не удалось обнаружить действительный файл ядра')
    }
  } catch (error) {
    message.error(`Проверка не удалась: ${error}`)
  } finally {
    loading.value = false
  }
}

// Получение директории данных приложения
const getAppDataPath = async () => {
  try {
    appDataPath.value = await appDataDir()
  } catch (error) {
    console.error('Не удалось получить директорию данных приложения:', error)
  }
}

// Слушатель событий прогресса скачивания
listen(
  'download-progress',
  (event: { payload: { status: string; progress: number; message: string } }) => {
    const { status, progress, message: msg } = event.payload
    downloadProgress.value = progress
    downloadMessage.value = msg

    if (status === 'completed') {
      downloading.value = false
      downloadError.value = null
      message.success('Скачивание ядра завершено!')
      // Обновление информации о версии
      infoStore.updateVersion()
    }
  },
)

onMounted(async () => {
  // Получение текущей версии
  await appStore.fetchAppVersion()
  // Проверка обновлений
  await checkUpdate()
  // Получение директории данных приложения
  await getAppDataPath()
  // Обновление информации о версии ядра
  await infoStore.updateVersion()
})
</script>

<style scoped>
.setting-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 16px 8px;
  display: flex;
  flex-direction: column;
  gap: 20px;
  animation: slide-up 0.4s ease;
}

.setting-card {
  border-radius: 16px;
  transition: all 0.3s ease;
  box-shadow: var(--shadow-light);
}

.setting-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-medium);
}

.card-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.card-title {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
  font-weight: 600;
  color: var(--n-text-color);
}

.card-icon {
  color: var(--primary-color);
}

.version-tag {
  font-weight: 500;
  padding: 0 12px;
  height: 28px;
}

.version-alert {
  border-radius: 10px;
  font-size: 14px;
}

.download-progress {
  margin: 10px 0;
  height: 36px;
  font-weight: 500;
}

.download-button {
  font-weight: 500;
  min-width: 140px;
  border-radius: 8px;
  transition: all 0.3s ease;
}

.download-button:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(64, 128, 255, 0.25);
}

.action-button {
  font-weight: 500;
  color: var(--n-text-color);
  transition: all 0.25s ease;
}

.action-button:hover:not(:disabled) {
  color: var(--primary-color);
  transform: translateY(-1px);
}

:deep(.n-switch) {
  --n-rail-color-active: var(--primary-color);
}

:deep(.n-radio-button) {
  border-radius: 8px;
}

:deep(.n-form-item-feedback) {
  font-size: 13px;
}

:deep(.n-tabs-nav) {
  background-color: transparent;
}

:deep(.n-tabs-tab) {
  font-weight: 500;
}

:deep(.n-tabs-tab.n-tabs-tab--active) {
  font-weight: 600;
  color: var(--primary-color);
}

.setting-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.setting-title {
  font-size: 14px;
  font-weight: 500;
}

.setting-desc {
  font-size: 12px;
  color: var(--text-color-3);
}

.about-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px;
  background-color: var(--card-color);
  border-radius: 4px;
}

.about-label {
  color: var(--text-color-2);
  font-size: 13px;
}

.about-value {
  color: var(--text-color-1);
  font-size: 13px;
  font-weight: 500;
}

.about-footer {
  margin-top: 24px;
  padding-top: 16px;
  border-top: 1px solid var(--divider-color);
}

.manual-path {
  font-family: monospace;
  background-color: var(--n-color-modal);
  padding: 8px;
  margin-top: 4px;
  border-radius: 4px;
  word-break: break-all;
}
</style>
