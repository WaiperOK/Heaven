#!/bin/bash

# Heaven AI Arena - Startup Script
# Автоматический запуск всей системы

set -e

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Функция для логирования
log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] $1${NC}"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING: $1${NC}"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $1${NC}"
}

# Проверка требований
check_requirements() {
    log "Проверка системных требований..."
    
    # Проверка Docker
    if ! command -v docker &> /dev/null; then
        error "Docker не установлен. Пожалуйста, установите Docker."
        exit 1
    fi
    
    # Проверка Docker Compose
    if ! command -v docker-compose &> /dev/null; then
        error "Docker Compose не установлен. Пожалуйста, установите Docker Compose."
        exit 1
    fi
    
    # Проверка Rust (для разработки)
    if ! command -v cargo &> /dev/null; then
        warn "Rust не установлен. Некоторые функции разработки могут быть недоступны."
    fi
    
    log "Все требования выполнены ✓"
}

# Создание необходимых директорий
create_directories() {
    log "Создание директорий..."
    
    mkdir -p data/logs
    mkdir -p data/uploads
    mkdir -p data/datasets
    mkdir -p models/llm
    mkdir -p models/cache
    mkdir -p models/hf
    
    log "Директории созданы ✓"
}

# Проверка переменных окружения
setup_environment() {
    log "Настройка окружения..."
    
    # Создаем .env файл если не существует
    if [ ! -f .env ]; then
        cat > .env << EOF
# Heaven AI Arena Configuration
DATABASE_URL=postgresql://heaven:heaven_pass@postgres:5432/heaven
REDIS_URL=redis://redis:6379
LLM_SERVICE_URL=http://llm_service:8000
ARENA_CORE_URL=http://arena_core:8080
DEFAULT_MODEL=llama2:7b
CUDA_VISIBLE_DEVICES=0
TRANSFORMERS_CACHE=/app/models/cache
HF_HOME=/app/models/hf
API_KEY=heaven_api_key
EOF
        log "Создан файл .env с настройками по умолчанию"
    fi
    
    log "Окружение настроено ✓"
}

# Сборка Docker образов
build_images() {
    log "Сборка Docker образов..."
    
    # Сборка всех сервисов
    docker-compose build --parallel
    
    log "Образы собраны ✓"
}

# Запуск сервисов
start_services() {
    log "Запуск сервисов..."
    
    # Запуск всех сервисов
    docker-compose up -d
    
    log "Сервисы запущены ✓"
}

# Проверка здоровья сервисов
check_health() {
    log "Проверка здоровья сервисов..."
    
    # Ждем запуска сервисов
    sleep 10
    
    # Проверяем PostgreSQL
    if docker-compose exec postgres pg_isready -U heaven -d heaven > /dev/null 2>&1; then
        log "PostgreSQL: Healthy ✓"
    else
        warn "PostgreSQL: Not ready"
    fi
    
    # Проверяем Redis
    if docker-compose exec redis redis-cli ping | grep -q "PONG"; then
        log "Redis: Healthy ✓"
    else
        warn "Redis: Not ready"
    fi
    
    # Проверяем LLM Service
    if curl -s -f http://localhost:8000/health > /dev/null 2>&1; then
        log "LLM Service: Healthy ✓"
    else
        warn "LLM Service: Not ready (может потребоваться больше времени)"
    fi
    
    # Проверяем No Code UI
    if curl -s -f http://localhost:8501 > /dev/null 2>&1; then
        log "No Code UI: Healthy ✓"
    else
        warn "No Code UI: Not ready"
    fi
    
    # Проверяем Arena Core
    if curl -s -f http://localhost:8080 > /dev/null 2>&1; then
        log "Arena Core: Healthy ✓"
    else
        warn "Arena Core: Not ready"
    fi
}

# Главная функция
main() {
    echo -e "${BLUE}"
    echo "================================="
    echo "   Heaven AI Arena Startup      "
    echo "================================="
    echo -e "${NC}"
    
    check_requirements
    create_directories
    setup_environment
    build_images
    start_services
    check_health
    
    echo -e "${GREEN}"
    echo "================================="
    echo "   Heaven AI Arena запущена!    "
    echo "================================="
    echo -e "${NC}"
    
    echo "Доступные сервисы:"
    echo "🌌 No Code UI:    http://localhost:8501"
    echo "🤖 LLM Service:   http://localhost:8000"
    echo "🎮 Arena Core:    http://localhost:8080"
    echo "🗄️  PostgreSQL:   localhost:5432"
    echo "🗃️  Redis:        localhost:6379"
    echo ""
    echo "Для просмотра логов: docker-compose logs -f"
    echo "Для остановки:       docker-compose down"
    echo "Для полной очистки:  docker-compose down -v"
    echo ""
    echo "Документация: README.md"
    echo "Поддержка: https://github.com/heaven-ai-arena/issues"
}

# Обработка аргументов командной строки
case "${1:-}" in
    "stop")
        log "Остановка сервисов..."
        docker-compose down
        log "Сервисы остановлены ✓"
        ;;
    "restart")
        log "Перезапуск сервисов..."
        docker-compose restart
        log "Сервисы перезапущены ✓"
        ;;
    "logs")
        docker-compose logs -f
        ;;
    "clean")
        log "Очистка системы..."
        docker-compose down -v
        docker system prune -f
        log "Система очищена ✓"
        ;;
    "dev")
        log "Запуск в режиме разработки..."
        export COMPOSE_FILE=docker-compose.yml:docker-compose.dev.yml
        main
        ;;
    "help"|"-h"|"--help")
        echo "Heaven AI Arena - Startup Script"
        echo ""
        echo "Использование: $0 [COMMAND]"
        echo ""
        echo "Команды:"
        echo "  start   - Запустить все сервисы (по умолчанию)"
        echo "  stop    - Остановить все сервисы"
        echo "  restart - Перезапустить все сервисы"
        echo "  logs    - Показать логи всех сервисов"
        echo "  clean   - Остановить и очистить все данные"
        echo "  dev     - Запустить в режиме разработки"
        echo "  help    - Показать эту справку"
        ;;
    *)
        main
        ;;
esac 