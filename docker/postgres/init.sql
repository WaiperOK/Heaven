-- Инициализация базы данных Heaven AI Arena

-- Создаем схему для таблиц
CREATE SCHEMA IF NOT EXISTS heaven;

-- Настраиваем пользователя
ALTER USER heaven CREATEDB;

-- Создаем расширения
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";

-- Создаем базовые таблицы (остальные будут созданы SQLAlchemy)
CREATE TABLE IF NOT EXISTS training_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    status VARCHAR(50) DEFAULT 'pending',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    model_name VARCHAR(255),
    task_type VARCHAR(100),
    config JSONB,
    progress FLOAT DEFAULT 0.0
);

CREATE TABLE IF NOT EXISTS request_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    model_name VARCHAR(255),
    endpoint VARCHAR(255),
    request_data JSONB,
    response_data JSONB,
    duration_ms INTEGER,
    status_code INTEGER
);

-- Индексы для оптимизации запросов
CREATE INDEX IF NOT EXISTS idx_training_tasks_status ON training_tasks(status);
CREATE INDEX IF NOT EXISTS idx_training_tasks_created_at ON training_tasks(created_at);
CREATE INDEX IF NOT EXISTS idx_request_logs_timestamp ON request_logs(timestamp);
CREATE INDEX IF NOT EXISTS idx_request_logs_model_name ON request_logs(model_name);

-- Настройки базы данных
ALTER DATABASE heaven SET timezone TO 'UTC';

-- Создаем таблицу для хранения конфигурации
CREATE TABLE IF NOT EXISTS system_config (
    key VARCHAR(255) PRIMARY KEY,
    value TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Начальная конфигурация
INSERT INTO system_config (key, value) VALUES 
('system_version', '0.1.0'),
('max_concurrent_training', '3'),
('default_model', 'llama2:7b'),
('arena_max_agents', '10')
ON CONFLICT (key) DO NOTHING;

-- Создаем функцию для обновления updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Создаем триггер для автоматического обновления updated_at
DROP TRIGGER IF EXISTS update_system_config_updated_at ON system_config;
CREATE TRIGGER update_system_config_updated_at
    BEFORE UPDATE ON system_config
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Сообщение об успешной инициализации
SELECT 'Heaven AI Arena database initialized successfully!' as message; 