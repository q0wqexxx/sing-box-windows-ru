import type { Emitter } from 'mitt'
import mitt from 'mitt'

// Определение интерфейса прогресса загрузки
interface DownloadProgress {
  status: 'checking' | 'found' | 'downloading' | 'extracting' | 'completed'
  progress: number
  message: string
}

// Определение интерфейса информации об обновлении
interface UpdateInfo {
  latest_version: string
  download_url: string
  has_update: boolean
}

// Определение типов событий
type Events = {
  'process-status': void
  'download-progress': DownloadProgress
  'proxy-mode-changed': void
  'window-minimize': void
  'window-hide': void
  'window-show': void
  'window-restore': void
  'refresh-tray-menu': void
  'update-available': UpdateInfo
  error: string
}

// Создание экземпляра шины событий
const emitter: Emitter<Events> = mitt<Events>()

export default emitter
