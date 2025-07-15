# Dockerfile для Godot Viewer
FROM ubuntu:22.04

# Устанавливаем необходимые пакеты
RUN apt-get update && apt-get install -y \
    wget \
    unzip \
    xvfb \
    x11-apps \
    libgl1 \
    libglu1-mesa \
    libasound2 \
    pulseaudio \
    && rm -rf /var/lib/apt/lists/*

# Скачиваем Godot headless
RUN wget https://downloads.tuxfamily.org/godotengine/4.2/Godot_v4.2-stable_linux.x86_64.zip \
    && unzip Godot_v4.2-stable_linux.x86_64.zip \
    && mv Godot_v4.2-stable_linux.x86_64 /usr/local/bin/godot \
    && chmod +x /usr/local/bin/godot \
    && rm Godot_v4.2-stable_linux.x86_64.zip

# Создаем рабочую директорию
WORKDIR /app

# Копируем проект Godot
COPY . .

# Импортируем проект 
RUN godot --headless --import

# Экспортируем как standalone приложение
RUN mkdir -p build && \
    godot --headless --export-release "Linux/X11" build/heaven_viewer

# Открываем порт для веб-экспорта (если нужно)
EXPOSE 8080

# Команда запуска
CMD ["./build/heaven_viewer"]