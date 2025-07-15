"""
Heaven AI Arena - No Code UI
Веб-интерфейс для дообучения моделей без кода
"""

import streamlit as st
import pandas as pd
import numpy as np
import requests
import time
import json
import os
from datetime import datetime
from pathlib import Path
import plotly.express as px
import plotly.graph_objects as go
from streamlit_option_menu import option_menu
from sqlalchemy import create_engine, text
import uuid

# Конфигурация
LLM_SERVICE_URL = os.getenv("LLM_SERVICE_URL", "http://localhost:8000")
ARENA_CORE_URL = os.getenv("ARENA_CORE_URL", "http://localhost:8080")
DATABASE_URL = os.getenv("DATABASE_URL", "postgresql://heaven:heaven_pass@postgres:5432/heaven")

# Настройка страницы
st.set_page_config(
    page_title="Heaven AI Arena - No Code Training",
    page_icon="🌌",
    layout="wide",
    initial_sidebar_state="expanded"
)

# Стили
st.markdown("""
<style>
.main-header {
    font-size: 3rem;
    font-weight: bold;
    color: #4A90E2;
    text-align: center;
    margin-bottom: 2rem;
}

.metric-card {
    background: #f0f2f6;
    padding: 1rem;
    border-radius: 10px;
    border-left: 4px solid #4A90E2;
    margin: 0.5rem 0;
}

.success-box {
    background: #d4edda;
    color: #155724;
    padding: 1rem;
    border-radius: 5px;
    border-left: 4px solid #28a745;
}

.error-box {
    background: #f8d7da;
    color: #721c24;
    padding: 1rem;
    border-radius: 5px;
    border-left: 4px solid #dc3545;
}

.warning-box {
    background: #fff3cd;
    color: #856404;
    padding: 1rem;
    border-radius: 5px;
    border-left: 4px solid #ffc107;
}
</style>
""", unsafe_allow_html=True)

# Заголовок
st.markdown('<h1 class="main-header">🌌 Heaven AI Arena</h1>', unsafe_allow_html=True)
st.markdown('<p style="text-align: center; font-size: 1.2rem; color: #666;">No Code Training Interface</p>', unsafe_allow_html=True)

# Функции для работы с API
@st.cache_data(ttl=30)
def get_llm_service_health():
    """Проверяет здоровье LLM сервиса"""
    try:
        response = requests.get(f"{LLM_SERVICE_URL}/health", timeout=5)
        return response.json() if response.status_code == 200 else None
    except:
        return None

@st.cache_data(ttl=60)
def get_available_models():
    """Получает список доступных моделей"""
    try:
        response = requests.get(f"{LLM_SERVICE_URL}/models", timeout=10)
        return response.json() if response.status_code == 200 else []
    except:
        return []

def start_training(training_config):
    """Запускает обучение модели"""
    try:
        response = requests.post(
            f"{LLM_SERVICE_URL}/train",
            json=training_config,
            timeout=30
        )
        return response.json() if response.status_code == 200 else None
    except Exception as e:
        st.error(f"Ошибка при запуске обучения: {e}")
        return None

def get_training_status(task_id):
    """Получает статус обучения"""
    try:
        response = requests.get(f"{LLM_SERVICE_URL}/training/{task_id}", timeout=10)
        return response.json() if response.status_code == 200 else None
    except:
        return None

def test_model_generation(model_name, prompt):
    """Тестирует генерацию текста моделью"""
    try:
        response = requests.post(
            f"{LLM_SERVICE_URL}/generate",
            json={
                "model": model_name,
                "prompt": prompt,
                "max_tokens": 100,
                "temperature": 0.7
            },
            timeout=30
        )
        return response.json() if response.status_code == 200 else None
    except Exception as e:
        st.error(f"Ошибка при тестировании модели: {e}")
        return None

# Боковое меню
with st.sidebar:
    selected = option_menu(
        "Главное меню",
        ["Панель управления", "Обучение моделей", "Датасеты", "Тестирование", "Мониторинг"],
        icons=["speedometer2", "cpu", "database", "play-circle", "graph-up"],
        menu_icon="cast",
        default_index=0,
        styles={
            "container": {"padding": "0!important", "background-color": "#fafafa"},
            "icon": {"color": "#4A90E2", "font-size": "18px"},
            "nav-link": {"font-size": "16px", "text-align": "left", "margin": "0px", "--hover-color": "#eee"},
            "nav-link-selected": {"background-color": "#4A90E2"},
        }
    )

# Проверка подключения к сервисам
health_status = get_llm_service_health()
if health_status:
    st.sidebar.success(f"✅ LLM Service: {health_status['status']}")
    st.sidebar.info(f"Device: {health_status['device']}")
    st.sidebar.info(f"Models loaded: {health_status['models_loaded']}")
else:
    st.sidebar.error("❌ LLM Service недоступен")

# Панель управления
if selected == "Панель управления":
    st.header("📊 Панель управления")
    
    # Метрики
    col1, col2, col3, col4 = st.columns(4)
    
    with col1:
        st.metric("Доступных моделей", len(get_available_models()))
    
    with col2:
        st.metric("Активных задач", 0)  # TODO: Получить из БД
    
    with col3:
        st.metric("Завершенных обучений", 0)  # TODO: Получить из БД
    
    with col4:
        st.metric("Статус системы", "Онлайн" if health_status else "Офлайн")
    
    # Доступные модели
    st.subheader("🤖 Доступные модели")
    models = get_available_models()
    
    if models:
        df_models = pd.DataFrame(models)
        df_models['created_at'] = pd.to_datetime(df_models['created_at'])
        st.dataframe(df_models, use_container_width=True)
    else:
        st.warning("Модели не найдены")
    
    # Последние задачи обучения
    st.subheader("📈 Последние задачи обучения")
    # TODO: Получить из БД и отобразить таблицу
    st.info("Функция в разработке")

# Обучение моделей
elif selected == "Обучение моделей":
    st.header("🎯 Обучение моделей")
    
    # Форма для создания задачи обучения
    with st.form("training_form"):
        st.subheader("Настройки обучения")
        
        col1, col2 = st.columns(2)
        
        with col1:
            # Базовая модель
            available_models = get_available_models()
            model_names = [m['name'] for m in available_models if m['type'] in ['hf', 'ollama']]
            
            base_model = st.selectbox(
                "Базовая модель",
                options=model_names,
                help="Выберите базовую модель для дообучения"
            )
            
            # Параметры обучения
            epochs = st.slider("Количество эпох", 1, 10, 3)
            learning_rate = st.number_input("Скорость обучения", value=2e-4, format="%.6f")
            batch_size = st.slider("Размер батча", 1, 32, 4)
            
        with col2:
            # LoRA параметры
            lora_rank = st.slider("LoRA Rank", 1, 128, 16)
            lora_alpha = st.slider("LoRA Alpha", 1, 256, 32)
            use_quantization = st.checkbox("Использовать квантизацию", value=True)
            
            # Выходная модель
            output_name = st.text_input(
                "Название новой модели",
                value=f"fine_tuned_{base_model}_{datetime.now().strftime('%Y%m%d_%H%M%S')}"
            )
        
        # Датасет
        st.subheader("Датасет")
        dataset_source = st.radio(
            "Источник датасета",
            ["Загрузить файл", "Использовать существующий"]
        )
        
        dataset_path = None
        if dataset_source == "Загрузить файл":
            uploaded_file = st.file_uploader(
                "Загрузите JSONL файл",
                type=['jsonl'],
                help="Файл должен содержать строки в формате JSON с полями 'instruction', 'input', 'output'"
            )
            
            if uploaded_file:
                # Сохраняем файл
                dataset_path = f"/app/data/uploads/{uploaded_file.name}"
                # TODO: Сохранить файл
                st.success(f"Файл {uploaded_file.name} загружен")
        else:
            dataset_path = st.text_input("Путь к датасету", value="/app/data/training_data.jsonl")
        
        # Предварительный просмотр конфигурации
        st.subheader("Предварительный просмотр конфигурации")
        config = {
            "model_name": base_model,
            "dataset_path": dataset_path,
            "output_name": output_name,
            "epochs": epochs,
            "learning_rate": learning_rate,
            "batch_size": batch_size,
            "lora_rank": lora_rank,
            "lora_alpha": lora_alpha,
            "use_quantization": use_quantization
        }
        
        st.json(config)
        
        # Кнопка запуска
        submitted = st.form_submit_button("🚀 Запустить обучение")
        
        if submitted:
            if not dataset_path:
                st.error("Выберите датасет для обучения")
            else:
                with st.spinner("Запуск обучения..."):
                    result = start_training(config)
                    
                    if result:
                        st.success(f"Обучение запущено! Task ID: {result['task_id']}")
                        st.session_state.current_task_id = result['task_id']
                    else:
                        st.error("Не удалось запустить обучение")
    
    # Статус текущего обучения
    if 'current_task_id' in st.session_state:
        st.subheader("📊 Статус обучения")
        
        task_id = st.session_state.current_task_id
        status = get_training_status(task_id)
        
        if status:
            col1, col2, col3 = st.columns(3)
            
            with col1:
                st.metric("Статус", status['status'])
            
            with col2:
                st.metric("Прогресс", f"{status.get('progress', 0):.1f}%")
            
            with col3:
                st.metric("Task ID", task_id)
            
            # Прогресс бар
            if status['status'] == 'running':
                progress = status.get('progress', 0) / 100
                st.progress(progress)
                
                # Автообновление
                if st.button("🔄 Обновить"):
                    st.experimental_rerun()
            
            elif status['status'] == 'completed':
                st.success("✅ Обучение завершено успешно!")
                
                # Кнопка для тестирования модели
                if st.button("🧪 Тестировать модель"):
                    st.session_state.test_model = status['output_name']
                    st.experimental_rerun()
            
            elif status['status'] == 'failed':
                st.error(f"❌ Обучение не удалось: {status.get('error_message', 'Unknown error')}")
        
        else:
            st.warning("Не удалось получить статус обучения")

# Датасеты
elif selected == "Датасеты":
    st.header("📚 Управление датасетами")
    
    # Загрузка датасета
    st.subheader("Загрузка нового датасета")
    uploaded_file = st.file_uploader(
        "Загрузите JSONL файл",
        type=['jsonl', 'json'],
        help="Файл должен содержать данные для обучения"
    )
    
    if uploaded_file:
        # Предварительный просмотр
        st.subheader("Предварительный просмотр")
        
        try:
            # Читаем первые несколько строк
            lines = uploaded_file.read().decode('utf-8').strip().split('\n')
            sample_data = []
            
            for i, line in enumerate(lines[:5]):  # Первые 5 строк
                try:
                    data = json.loads(line)
                    sample_data.append(data)
                except json.JSONDecodeError:
                    st.error(f"Ошибка парсинга JSON в строке {i+1}")
                    break
            
            if sample_data:
                df = pd.DataFrame(sample_data)
                st.dataframe(df, use_container_width=True)
                
                # Статистика
                st.subheader("Статистика датасета")
                col1, col2, col3 = st.columns(3)
                
                with col1:
                    st.metric("Всего записей", len(lines))
                
                with col2:
                    st.metric("Колонки", len(df.columns))
                
                with col3:
                    st.metric("Размер файла", f"{len(uploaded_file.read())} bytes")
                
                # Кнопка сохранения
                if st.button("💾 Сохранить датасет"):
                    # TODO: Сохранить файл в /app/data/
                    st.success("Датасет сохранен!")
        
        except Exception as e:
            st.error(f"Ошибка при обработке файла: {e}")
    
    # Список существующих датасетов
    st.subheader("Существующие датасеты")
    # TODO: Получить список файлов из /app/data/
    st.info("Функция в разработке")

# Тестирование
elif selected == "Тестирование":
    st.header("🧪 Тестирование моделей")
    
    # Выбор модели
    available_models = get_available_models()
    model_names = [m['name'] for m in available_models]
    
    selected_model = st.selectbox("Выберите модель для тестирования", model_names)
    
    # Настройки генерации
    col1, col2 = st.columns(2)
    
    with col1:
        temperature = st.slider("Temperature", 0.0, 2.0, 0.7)
        max_tokens = st.slider("Max Tokens", 1, 512, 100)
    
    with col2:
        top_p = st.slider("Top-p", 0.0, 1.0, 0.9)
        system_prompt = st.text_area("System Prompt (опционально)", height=100)
    
    # Ввод промпта
    prompt = st.text_area("Введите промпт для тестирования:", height=200)
    
    if st.button("🎯 Сгенерировать") and prompt:
        with st.spinner("Генерация..."):
            result = test_model_generation(selected_model, prompt)
            
            if result:
                st.subheader("Результат:")
                st.write(result['text'])
                
                # Метрики
                col1, col2, col3 = st.columns(3)
                
                with col1:
                    st.metric("Токенов использовано", result['tokens_used'])
                
                with col2:
                    st.metric("Время обработки", f"{result['processing_time_ms']} мс")
                
                with col3:
                    st.metric("Модель", result['model_name'])
            
            else:
                st.error("Не удалось сгенерировать текст")

# Мониторинг
elif selected == "Мониторинг":
    st.header("📈 Мониторинг системы")
    
    # Системные метрики
    if health_status:
        col1, col2 = st.columns(2)
        
        with col1:
            st.subheader("Состояние LLM сервиса")
            st.json(health_status)
        
        with col2:
            st.subheader("Использование ресурсов")
            # TODO: Получить метрики использования ресурсов
            st.info("Функция в разработке")
    
    # Статистика запросов
    st.subheader("Статистика запросов")
    # TODO: Получить данные из БД и построить графики
    
    # Пример графика
    sample_data = {
        'timestamp': pd.date_range(start='2024-01-01', periods=100, freq='H'),
        'requests': np.random.randint(1, 50, 100),
        'response_time': np.random.uniform(100, 5000, 100)
    }
    
    df = pd.DataFrame(sample_data)
    
    # График запросов
    fig_requests = px.line(df, x='timestamp', y='requests', title='Запросы в час')
    st.plotly_chart(fig_requests, use_container_width=True)
    
    # График времени ответа
    fig_response = px.line(df, x='timestamp', y='response_time', title='Время ответа (мс)')
    st.plotly_chart(fig_response, use_container_width=True)

# Футер
st.markdown("---")
st.markdown("© 2024 Heaven AI Arena Team. Все права защищены.")

# Автообновление для активных задач
if 'current_task_id' in st.session_state:
    time.sleep(1)  # Небольшая задержка для демонстрации
    # В реальном приложении можно использовать st.experimental_rerun() 