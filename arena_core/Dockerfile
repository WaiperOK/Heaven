# Используем официальный образ Rust
FROM rust:1.82-slim as builder

# Устанавливаем системные зависимости
RUN apt-get update && apt-get install -y \
    xvfb \
    x11-apps \
    pkg-config \
    libssl-dev \
    libasound2-dev \
    libudev-dev \
    libx11-6 \
    libxrandr2 \
    libxinerama1 \
    libxcursor1 \
    libxi6 \
    libgl1 \
    mesa-utils \
    xauth \
    && rm -rf /var/lib/apt/lists/*
RUN find / -name xvfb-run || true

# Создаем рабочую директорию
WORKDIR /app

# Копируем файлы проекта
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Собираем проект
RUN cargo build --release --no-default-features

# Финальный образ
FROM debian:bookworm-slim as stage-1

# Устанавливаем системные зависимости
RUN apt-get update && apt-get install -y \
    libssl3 \
    xvfb \
    x11-apps \
    libx11-6 \
    libxrandr2 \
    libxinerama1 \
    libxcursor1 \
    libxi6 \
    libgl1 \
    mesa-utils \
    libasound2 \
    && rm -rf /var/lib/apt/lists/*

# Создаем пользователя для запуска
RUN useradd -m -u 1000 heaven

# Создаем директории
RUN mkdir -p /app/models /app/data && \
    chown -R heaven:heaven /app

# Копируем собранный бинарный файл
COPY --from=builder /app/target/release/arena_core /app/arena_core
RUN chmod +x /app/arena_core

# Переключаемся на пользователя
USER heaven
WORKDIR /app

# Открываем порт
EXPOSE 8080

# Запускаем Xvfb вручную и экспортируем DISPLAY
CMD bash -c "Xvfb :99 -screen 0 1024x768x16 & export DISPLAY=:99 && ./arena_core" 