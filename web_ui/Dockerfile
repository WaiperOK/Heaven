# Базовый образ Node.js
FROM node:18-alpine

# Устанавливаем рабочую директорию
WORKDIR /app

# Копируем package.json и package-lock.json
COPY package*.json ./

# Устанавливаем зависимости
RUN npm install

# Копируем исходный код
COPY . .

# Собираем проект
RUN npm run build

# Устанавливаем serve для production
RUN npm install -g serve

# Открываем порт
EXPOSE 3000

# Команда запуска
CMD ["serve", "-s", "build", "-l", "3000"]