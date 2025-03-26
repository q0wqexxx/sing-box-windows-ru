<template>
  <div class="traffic-chart-container" ref="chartContainer">
    <canvas ref="chartCanvas" class="chart-canvas"></canvas>
    <div class="chart-labels">
      <div class="legend-item upload">
        <div class="legend-color"></div>
        <span>Скорость загрузки</span>
      </div>
      <div class="legend-item download">
        <div class="legend-color"></div>
        <span>Скорость скачивания</span>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref, defineProps, onMounted, onUnmounted, watch, computed } from 'vue'
import { useThemeVars } from 'naive-ui'
import { formatBandwidth } from '@/utils/index' // Импорт функции formatBandwidth

defineOptions({
  name: 'TrafficChart',
})

const props = defineProps({
  uploadSpeed: {
    type: Number,
    default: 0,
  },
  downloadSpeed: {
    type: Number,
    default: 0,
  },
})

const chartContainer = ref<HTMLDivElement | null>(null)
const chartCanvas = ref<HTMLCanvasElement | null>(null)
const themeVars = useThemeVars()

// Конфигурация графика
const MAX_DATA_POINTS = 60 // Максимальное количество точек данных
const uploadData = ref<number[]>([]) // Данные скорости загрузки
const downloadData = ref<number[]>([]) // Данные скорости скачивания
const timeLabels = ref<string[]>([]) // Метки времени

// Вычисление максимального значения
const maxValue = computed(() => {
  const uploadMax = Math.max(...uploadData.value, 0.1)
  const downloadMax = Math.max(...downloadData.value, 0.1)
  return Math.max(uploadMax, downloadMax) * 1.2 // Оставить 20% пространства
})

// Инициализация графика
const initChart = () => {
  if (!chartCanvas.value || !chartContainer.value) return

  const canvas = chartCanvas.value
  const container = chartContainer.value
  const { width, height } = container.getBoundingClientRect()

  // Установка размера canvas, учитывая плотность пикселей устройства для сохранения четкости
  const dpr = window.devicePixelRatio || 1
  canvas.width = width * dpr
  canvas.height = height * dpr
  canvas.style.width = `${width}px`
  canvas.style.height = `${height}px`

  // Инициализация массивов данных
  uploadData.value = Array(MAX_DATA_POINTS).fill(0)
  downloadData.value = Array(MAX_DATA_POINTS).fill(0)
  timeLabels.value = Array(MAX_DATA_POINTS).fill('')

  // Немедленное рисование пустого графика
  drawChart()
}

// Рисование графика
const drawChart = () => {
  if (!chartCanvas.value) return

  const canvas = chartCanvas.value
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const dpr = window.devicePixelRatio || 1
  const width = canvas.width
  const height = canvas.height
  const padding = { top: 30 * dpr, right: 20 * dpr, bottom: 40 * dpr, left: 80 * dpr } // Увеличение отступа слева

  // Очистка холста
  ctx.clearRect(0, 0, width, height)

  // Область рисования
  const chartWidth = width - padding.left - padding.right
  const chartHeight = height - padding.top - padding.bottom

  // Получение текущих цветов темы
  const bgColor = themeVars.value.bodyColor
  const textColor = themeVars.value.textColor2
  const gridColor = themeVars.value.borderColor
  const uploadColor = '#18A058' // Зеленый
  const downloadColor = '#2080F0' // Синий

  // Установка шрифта
  ctx.font = `${12 * dpr}px sans-serif`
  ctx.textAlign = 'right'
  ctx.textBaseline = 'middle'
  ctx.fillStyle = textColor

  // Рисование меток и сетки по оси Y
  const yAxisSteps = 5
  for (let i = 0; i <= yAxisSteps; i++) {
    const y = padding.top + chartHeight - (i / yAxisSteps) * chartHeight
    const value = (i / yAxisSteps) * maxValue.value

    // Преобразование значения в строку с подходящей единицей измерения (передача параметра в КБ)
    const formattedValue = formatBandwidth(value * 1024 * 1024) // Преобразование в КБ перед передачей в formatBandwidth

    // Добавление "/s" для обозначения единицы скорости
    const speedLabel = `${formattedValue}/s`

    // Рисование сетки
    ctx.beginPath()
    ctx.strokeStyle = gridColor
    ctx.lineWidth = 1
    ctx.moveTo(padding.left, y)
    ctx.lineTo(padding.left + chartWidth, y)
    ctx.stroke()

    // Рисование меток по оси Y
    ctx.fillText(speedLabel, padding.left - 10 * dpr, y)
  }

  // Рисование оси X
  ctx.beginPath()
  ctx.strokeStyle = gridColor
  ctx.lineWidth = 1
  ctx.moveTo(padding.left, padding.top + chartHeight)
  ctx.lineTo(padding.left + chartWidth, padding.top + chartHeight)
  ctx.stroke()

  // Рисование меток по оси X (отображение только части меток времени для избежания перегруженности)
  const labelInterval = Math.ceil(MAX_DATA_POINTS / 6) // Отображение около 6 меток
  for (let i = 0; i < MAX_DATA_POINTS; i += labelInterval) {
    if (timeLabels.value[i]) {
      const x = padding.left + (i / (MAX_DATA_POINTS - 1)) * chartWidth
      ctx.textAlign = 'center'
      ctx.fillText(timeLabels.value[i], x, padding.top + chartHeight + 20 * dpr)
    }
  }

  // Рисование кривой скорости загрузки
  if (uploadData.value.some((v) => v > 0)) {
    drawCurve(ctx, uploadData.value, uploadColor, padding, chartWidth, chartHeight, dpr)
  }

  // Рисование кривой скорости скачивания
  if (downloadData.value.some((v) => v > 0)) {
    drawCurve(ctx, downloadData.value, downloadColor, padding, chartWidth, chartHeight, dpr)
  }
}

// Функция рисования кривой
const drawCurve = (
  ctx: CanvasRenderingContext2D,
  data: number[],
  color: string,
  padding: { top: number; right: number; bottom: number; left: number },
  chartWidth: number,
  chartHeight: number,
  dpr: number,
) => {
  const max = maxValue.value || 0.1 // Избегаем деления на ноль

  // Рисование кривой
  ctx.beginPath()
  ctx.strokeStyle = color
  ctx.lineWidth = 2 * dpr
  ctx.lineJoin = 'round'

  data.forEach((value, index) => {
    const x = padding.left + (index / (MAX_DATA_POINTS - 1)) * chartWidth
    const y = padding.top + chartHeight - (value / max) * chartHeight

    if (index === 0) {
      ctx.moveTo(x, y)
    } else {
      ctx.lineTo(x, y)
    }
  })

  ctx.stroke()

  // Рисование градиентной области
  ctx.lineTo(padding.left + chartWidth, padding.top + chartHeight)
  ctx.lineTo(padding.left, padding.top + chartHeight)
  ctx.closePath()

  const gradient = ctx.createLinearGradient(0, padding.top, 0, padding.top + chartHeight)
  gradient.addColorStop(0, `${color}40`) // 40 - шестнадцатеричный код прозрачности
  gradient.addColorStop(1, `${color}05`) // 05 - шестнадцатеричный код прозрачности

  ctx.fillStyle = gradient
  ctx.fill()
}

// Обновление данных
const updateData = () => {
  // Сохранение данных в байтах для удобства последующей обработки
  const uploadSpeed = props.uploadSpeed
  const downloadSpeed = props.downloadSpeed

  // Удаление самых старых данных
  uploadData.value.shift()
  downloadData.value.shift()
  timeLabels.value.shift()

  // Добавление новых данных (сохранение значений в МБ для сохранения существующей логики)
  uploadData.value.push(uploadSpeed / 1024 / 1024)
  downloadData.value.push(downloadSpeed / 1024 / 1024)

  const now = new Date()
  const timeStr = `${now.getMinutes().toString().padStart(2, '0')}:${now.getSeconds().toString().padStart(2, '0')}`
  timeLabels.value.push(timeStr)

  // Перерисовка графика
  drawChart()
}

let updateTimer: number | null = null

// Запуск периодического обновления
const startUpdates = () => {
  if (updateTimer !== null) {
    clearInterval(updateTimer)
  }

  console.log('Запуск таймера обновления графика')
  updateTimer = setInterval(() => {
    updateData()
  }, 1000) as unknown as number
}

// Сброс и обновление графика
const resetAndRefresh = () => {
  console.log('Сброс и обновление графика')

  // Очистка всех данных
  uploadData.value = Array(MAX_DATA_POINTS).fill(0)
  downloadData.value = Array(MAX_DATA_POINTS).fill(0)
  timeLabels.value = Array(MAX_DATA_POINTS).fill('')

  // Обеспечение повторного получения размеров контейнера
  if (chartContainer.value && chartCanvas.value) {
    const { width, height } = chartContainer.value.getBoundingClientRect()
    const dpr = window.devicePixelRatio || 1

    // Сброс размеров холста для принудительного повторного рендеринга
    chartCanvas.value.width = width * dpr
    chartCanvas.value.height = height * dpr
    chartCanvas.value.style.width = `${width}px`
    chartCanvas.value.style.height = `${height}px`
  }

  // Повторная инициализация графика
  setTimeout(() => {
    initChart()
    // Немедленное обновление данных для отображения текущего состояния
    updateData()
    // Обеспечение работы таймера обновления
    if (updateTimer === null) {
      startUpdates()
    }
  }, 50)
}

// Инициализация при монтировании компонента
onMounted(() => {
  // Задержка выполнения для обеспечения полной отрисовки DOM
  setTimeout(() => {
    initChart()
    startUpdates()
  }, 100)

  // Добавление слушателя событий изменения размера окна
  window.addEventListener('resize', handleResize)
})

// Очистка при размонтировании компонента
onUnmounted(() => {
  if (updateTimer !== null) {
    clearInterval(updateTimer)
    updateTimer = null
  }

  window.removeEventListener('resize', handleResize)
})

// Слушатель изменений темы
watch(themeVars, () => {
  console.log('Изменение темы, перерисовка графика')
  drawChart()
})

// Обработка изменения размера окна
const handleResize = () => {
  if (chartContainer.value && chartCanvas.value) {
    console.log('Изменение размера окна, повторная отрисовка графика')
    initChart()
  }
}
</script>

<style scoped>
.traffic-chart-container {
  position: relative;
  height: 100%;
  width: 100%;
  display: flex;
  flex-direction: column;
}

.chart-canvas {
  flex-grow: 1;
  width: 100%;
  height: 100%;
}

.chart-labels {
  position: absolute;
  top: 10px;
  right: 20px;
  display: flex;
  gap: 20px;
  z-index: 1;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--n-text-color-1);
}

.legend-color {
  width: 12px;
  height: 12px;
  border-radius: 3px;
}

.upload .legend-color {
  background-color: #18a058;
}

.download .legend-color {
  background-color: #2080f0;
}
</style>
