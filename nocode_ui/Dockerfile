# Базовый образ Python
FROM python:3.11-slim

# Устанавливаем системные зависимости
RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Устанавливаем рабочую директорию
WORKDIR /app

# Копируем requirements.txt
COPY requirements.txt .

# Устанавливаем Python зависимости
RUN pip install --no-cache-dir -r requirements.txt

# Копируем код приложения
COPY . .

# Создаем директории для данных
RUN mkdir -p /app/data /app/uploads

# Открываем порт
EXPOSE 8501

# Команда запуска
CMD ["streamlit", "run", "main.py", "--server.address=0.0.0.0", "--server.port=8501"] 