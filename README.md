# Sing-Box Windows

<div align="center">
    <img src="./src/assets/icon.png" alt="Sing-Box Windows Logo" width="128" height="128" />
    <h1>Sing-Box Windows</h1>
    <p>🚀 Элегантный клиент Sing-Box для Windows с графическим интерфейсом</p>
    <p>
        <img src="https://img.shields.io/github/license/xinggaoya/sing-box-windows" alt="license" />
        <img src="https://img.shields.io/github/stars/xinggaoya/sing-box-windows" alt="stars" />
        <img src="https://img.shields.io/github/downloads/xinggaoya/sing-box-windows/total" alt="downloads" />
        <img src="https://img.shields.io/github/v/release/xinggaoya/sing-box-windows" alt="release" />
    </p>
</div>

## 🎯 Особенности

- 🖥️ Современный пользовательский интерфейс, основанный на [Tauri 2.0](https://tauri.app/) и [Vue 3](https://vuejs.org/)
- 🌙 Поддержка светлой/темной темы, автоматическое следование за системными настройками
- 🔄 Поддержка импорта и автоматического обновления различных подписок
  - Поддержка форматов Base64/JSON/SIP008 и других
  - Поддержка автоматической проверки обновлений и планового обновления
- 🌐 Полная поддержка различных режимов прокси
  - Системный прокси режим
  - TUN режим (требуются права администратора)
  - Разделение по правилам
- 📊 Богатые функции статистики
  - Мониторинг трафика в реальном времени
  - Графики использования трафика
  - Тестирование скорости соединения
- 🔍 Полная система логирования
  - Просмотр логов в реальном времени
  - Настройка уровня логирования
  - Экспорт логов в файл
- ⚡ Оптимизированная производительность
  - Низкое потребление памяти (около 50-100MB)
  - Быстрый запуск и отклик
  - Автоматическая работа в фоновом режиме
- 🔒 Безопасные функции
  - Автоматическая защита от утечек DNS
  - Автоматическое разделение по правилам
  - Безопасное хранение конфигурации

## 📸 Превью

<img src="./public/image.png" alt="sing-box-windows превью" width="800">

## 🚀 Быстрый старт

### Системные требования

- Windows 10 1809 или выше
- Не менее 2GB свободной памяти
- Не менее 200MB свободного места на диске

### Установка и запуск

1. Скачайте последнюю версию с [Releases](https://github.com/xinggaoya/sing-box-windows/releases) страницы
2. Запустите установщик (поддерживается автоматическое обновление)
3. При первом запуске будет выполнена необходимая настройка

### Основное использование

1. При первом использовании загрузите и установите ядро Sing-Box в разделе "Настройки"
2. В разделе "Подписки" добавьте или импортируйте свои подписки
   - Поддерживается ввод ссылки напрямую
   - Поддерживается импорт из буфера обмена
   - Поддерживается импорт из файла конфигурации
3. В разделе "Главная" выберите узел и подключитесь
   - Поддерживается быстрое переключение узлов
   - Поддерживается тестирование задержки узлов
   - Поддерживается управление группами узлов

> Подсказка: При первом использовании TUN режима программа запросит права администратора и автоматически настроит системные параметры

### Расширенные функции

- **Настройка правил**: поддержка пользовательских правил разделения
- **Быстрые действия**: поддержка быстрых действий через системный трей
- **Резервное копирование конфигурации**: поддержка экспорта и восстановления конфигурации
- **Автоматизация**: поддержка автозапуска и автоматического подключения при старте системы

## 🛠️ Руководство по разработке

### Требования к окружению

- [Node.js](https://nodejs.org/) версии 18.0 или выше
- [Rust](https://www.rust-lang.org/) последняя стабильная версия
- [Visual Studio](https://visualstudio.microsoft.com/) 2019 или выше (с установленными инструментами для разработки на C++)
- [Git](https://git-scm.com/) последняя версия
- [pnpm](https://pnpm.io/) пакетный менеджер

### Локальная разработка

```bash
# Клонирование проекта
git clone https://github.com/xinggaoya/sing-box-windows.git
cd sing-box-windows

# Установка зависимостей
pnpm install

# Запуск сервера разработки
pnpm tauri dev

# Сборка производственной версии
pnpm tauri build
```

### Дополнительная документация по разработке

Подробную документацию по разработке можно найти в [документации по разработке](./docs/development.md), которая включает в себя структуру проекта, основные функциональные модули, стандарты разработки и многое другое.

### Структура проекта

```
sing-box-windows/
├── src/                # Исходный код фронтенда
│   ├── assets/        # Статические ресурсы
│   ├── components/    # Общие компоненты
│   ├── router/        # Конфигурация маршрутизации
│   ├── stores/        # Управление состоянием
│   ├── utils/         # Вспомогательные функции
│   └── views/         # Компоненты страниц
├── src-tauri/         # Исходный код бэкенда на Rust
│   ├── src/           # Исходный код
│   └── Cargo.toml     # Конфигурация зависимостей Rust
└── package.json       # Конфигурация проекта
```

## 📦 Технологический стек

- 🎯 [Tauri 2.0](https://tauri.app/) - современный кроссплатформенный фреймворк для приложений
- ⚡ [Vue 3](https://vuejs.org/) - реактивный фронтенд фреймворк
- 🎨 [Naive UI](https://www.naiveui.com/) - высококачественная библиотека компонентов для Vue 3
- 📊 [Canvas API](https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API) - высокопроизводительная графическая отрисовка
- 🦀 [Rust](https://www.rust-lang.org/) - высокопроизводительный системный язык программирования
- 🔧 [TypeScript](https://www.typescriptlang.org/) - типобезопасный JavaScript

## 🤝 Руководство по вкладу

Мы приветствуем любые формы вклада, включая, но не ограничиваясь:

- 🐛 Сообщения об ошибках и предложения
- 📝 Улучшение документации
- 🔧 Исправление кода
- ✨ Разработка новых функций
- 🌍 Поддержка многоязычности

Процесс вклада:

1. Форкните этот репозиторий
2. Создайте ветку для новой функции (`git checkout -b feature/AmazingFeature`)
3. Внесите изменения (`git commit -m 'Add some AmazingFeature'`)
4. Запушьте ветку (`git push origin feature/AmazingFeature`)
5. Создайте Pull Request

## 📄 Лицензия

Этот проект лицензирован под [MIT лицензией](LICENSE).

## 📮 Контакты

- 📧 Email: [xinggaoya@qq.com](mailto:xinggaoya@qq.com)
- 🐛 Сообщения об ошибках: [GitHub Issues](https://github.com/xinggaoya/sing-box-windows/issues)
- 💬 Обсуждения: [GitHub Discussions](https://github.com/xinggaoya/sing-box-windows/discussions)

## ⭐ Благодарности

- [sing-box](https://github.com/SagerNet/sing-box) - основной движок прокси
- [Tauri](https://tauri.app/) - фреймворк для приложений
- [Vue](https://vuejs.org/) - фронтенд фреймворк
- [Naive UI](https://www.naiveui.com/) - библиотека UI компонентов
- [Сообщество участников](https://github.com/xinggaoya/sing-box-windows/graphs/contributors)

---

Если этот проект был вам полезен, пожалуйста, поставьте звезду ⭐️
