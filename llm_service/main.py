"""
Heaven AI Arena - LLM Service
Сервис для работы с локальными LLM моделями, дообучения и inference
"""

import asyncio
import logging
import os
import time
from contextlib import asynccontextmanager
from typing import Dict, List, Optional, Any
from pathlib import Path
import uuid

from fastapi import FastAPI, HTTPException, BackgroundTasks, Depends, status
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from pydantic import BaseModel, Field
import uvicorn
from transformers import AutoTokenizer, AutoModelForCausalLM
import torch
from peft import PeftModel, PeftConfig
import redis
from sqlalchemy import create_engine, Column, String, DateTime, Integer, Float, Text, Boolean
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker, Session
from datetime import datetime
import json
import ollama

# Настройка логирования
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# Конфигурация
class Config:
    # Директории
    MODELS_DIR = Path("/app/models")
    DATA_DIR = Path("/app/data")
    
    # База данных
    DATABASE_URL = os.getenv("DATABASE_URL", "postgresql://heaven:heaven_pass@postgres:5432/heaven")
    
    # Redis
    REDIS_URL = os.getenv("REDIS_URL", "redis://redis:6379")
    
    # Модели
    DEFAULT_MODEL = os.getenv("DEFAULT_MODEL", "llama3.2:1b")
    HF_CACHE_DIR = MODELS_DIR / "hf"
    
    # Устройство
    DEVICE = "cuda" if torch.cuda.is_available() else "cpu"
    
    # Лимиты
    MAX_TOKENS = 512
    MAX_CONCURRENT_REQUESTS = 10
    REQUEST_TIMEOUT = 30
    
    # Безопасность
    API_KEY = os.getenv("API_KEY", "heaven_api_key")

config = Config()

# Создаем директории
config.MODELS_DIR.mkdir(parents=True, exist_ok=True)
config.DATA_DIR.mkdir(parents=True, exist_ok=True)
config.HF_CACHE_DIR.mkdir(parents=True, exist_ok=True)

# Модели данных
class GenerateRequest(BaseModel):
    model: str = Field(default=config.DEFAULT_MODEL, description="Название модели")
    prompt: str = Field(..., description="Входной промпт")
    max_tokens: int = Field(default=150, le=config.MAX_TOKENS, description="Максимальное количество токенов")
    temperature: float = Field(default=0.7, ge=0.0, le=2.0, description="Температура генерации")
    top_p: float = Field(default=0.9, ge=0.0, le=1.0, description="Top-p sampling")
    stop_sequences: Optional[List[str]] = Field(default=None, description="Стоп-последовательности")
    system_prompt: Optional[str] = Field(default=None, description="Системный промпт")

class GenerateResponse(BaseModel):
    text: str
    tokens_used: int
    processing_time_ms: int
    model_name: str
    request_id: str

class TrainingRequest(BaseModel):
    model_name: str = Field(..., description="Название базовой модели")
    dataset_path: str = Field(..., description="Путь к датасету")
    output_name: str = Field(..., description="Название выходной модели")
    epochs: int = Field(default=3, ge=1, le=10, description="Количество эпох")
    learning_rate: float = Field(default=2e-4, gt=0.0, description="Скорость обучения")
    batch_size: int = Field(default=4, ge=1, le=32, description="Размер батча")
    lora_rank: int = Field(default=16, ge=1, le=128, description="Ранг LoRA")
    lora_alpha: int = Field(default=32, ge=1, le=256, description="Альфа LoRA")
    use_quantization: bool = Field(default=True, description="Использовать квантизацию")

class TrainingResponse(BaseModel):
    task_id: str
    status: str
    message: str

class ModelInfo(BaseModel):
    name: str
    type: str  # "hf", "ollama", "peft"
    size: str
    description: str
    created_at: datetime
    is_available: bool

# База данных
Base = declarative_base()

class TrainingTask(Base):
    __tablename__ = "training_tasks"
    
    id = Column(String, primary_key=True, default=lambda: str(uuid.uuid4()))
    model_name = Column(String, nullable=False)
    dataset_path = Column(String, nullable=False)
    output_name = Column(String, nullable=False)
    status = Column(String, nullable=False, default="pending")  # pending, running, completed, failed
    progress = Column(Float, default=0.0)
    created_at = Column(DateTime, default=datetime.utcnow)
    started_at = Column(DateTime, nullable=True)
    completed_at = Column(DateTime, nullable=True)
    error_message = Column(Text, nullable=True)
    config = Column(Text, nullable=True)  # JSON конфигурация
    metrics = Column(Text, nullable=True)  # JSON метрики

class RequestLog(Base):
    __tablename__ = "request_logs"
    
    id = Column(String, primary_key=True, default=lambda: str(uuid.uuid4()))
    model_name = Column(String, nullable=False)
    prompt_length = Column(Integer, nullable=False)
    tokens_generated = Column(Integer, nullable=False)
    processing_time_ms = Column(Integer, nullable=False)
    timestamp = Column(DateTime, default=datetime.utcnow)
    success = Column(Boolean, default=True)
    error_message = Column(Text, nullable=True)

# Создаем подключение к БД
engine = create_engine(config.DATABASE_URL)
Base.metadata.create_all(bind=engine)
SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)

def get_db():
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()

# Менеджер моделей
class ModelManager:
    def __init__(self):
        self.models: Dict[str, Any] = {}
        self.tokenizers: Dict[str, Any] = {}
        self.ollama_client = ollama.Client(host="http://ollama:11434")
        self.redis_client = redis.Redis.from_url(config.REDIS_URL)
        
    async def load_model(self, model_name: str, force_reload: bool = False):
        """Загружает модель в память"""
        if model_name in self.models and not force_reload:
            return self.models[model_name]
        
        try:
            # Проверяем, есть ли модель в Ollama
            if self.is_ollama_model(model_name):
                logger.info(f"Using Ollama model: {model_name}")
                return "ollama"
            
            # Проверяем, есть ли PEFT модель
            peft_path = config.MODELS_DIR / "llm" / model_name
            if peft_path.exists():
                logger.info(f"Loading PEFT model: {model_name}")
                return await self.load_peft_model(str(peft_path))
            
            # Загружаем HuggingFace модель
            logger.info(f"Loading HuggingFace model: {model_name}")
            return await self.load_hf_model(model_name)
            
        except Exception as e:
            logger.error(f"Failed to load model {model_name}: {e}")
            raise HTTPException(status_code=500, detail=f"Failed to load model: {e}")
    
    def is_ollama_model(self, model_name: str) -> bool:
        """Проверяет, доступна ли модель в Ollama"""
        try:
            models = self.ollama_client.list()
            return any(model['name'] == model_name for model in models['models'])
        except:
            return False
    
    async def load_hf_model(self, model_name: str):
        """Загружает модель HuggingFace"""
        try:
            tokenizer = AutoTokenizer.from_pretrained(
                model_name,
                cache_dir=config.HF_CACHE_DIR,
                trust_remote_code=True
            )
            
            model = AutoModelForCausalLM.from_pretrained(
                model_name,
                cache_dir=config.HF_CACHE_DIR,
                torch_dtype=torch.float16 if config.DEVICE == "cuda" else torch.float32,
                device_map="auto" if config.DEVICE == "cuda" else None,
                trust_remote_code=True
            )
            
            self.models[model_name] = model
            self.tokenizers[model_name] = tokenizer
            
            logger.info(f"Successfully loaded HF model: {model_name}")
            return model
            
        except Exception as e:
            logger.error(f"Failed to load HF model {model_name}: {e}")
            raise
    
    async def load_peft_model(self, model_path: str):
        """Загружает PEFT модель"""
        try:
            config_peft = PeftConfig.from_pretrained(model_path)
            
            # Загружаем базовую модель
            base_model = AutoModelForCausalLM.from_pretrained(
                config_peft.base_model_name_or_path,
                cache_dir=config.HF_CACHE_DIR,
                torch_dtype=torch.float16 if config.DEVICE == "cuda" else torch.float32,
                device_map="auto" if config.DEVICE == "cuda" else None,
                trust_remote_code=True
            )
            
            # Загружаем адаптер
            model = PeftModel.from_pretrained(base_model, model_path)
            
            tokenizer = AutoTokenizer.from_pretrained(
                config_peft.base_model_name_or_path,
                cache_dir=config.HF_CACHE_DIR,
                trust_remote_code=True
            )
            
            model_name = Path(model_path).name
            self.models[model_name] = model
            self.tokenizers[model_name] = tokenizer
            
            logger.info(f"Successfully loaded PEFT model: {model_name}")
            return model
            
        except Exception as e:
            logger.error(f"Failed to load PEFT model {model_path}: {e}")
            raise
    
    async def generate_text(self, request: GenerateRequest) -> GenerateResponse:
        """Генерирует текст с помощью модели"""
        request_id = str(uuid.uuid4())
        start_time = time.time()
        
        try:
            # Проверяем, есть ли результат в кеше
            cache_key = f"generate:{hash(request.prompt + request.model)}"
            cached_result = self.redis_client.get(cache_key)
            
            if cached_result:
                logger.info(f"Using cached result for request {request_id}")
                return GenerateResponse.parse_raw(cached_result)
            
            # Загружаем модель
            model = await self.load_model(request.model)
            
            # Генерируем текст
            if model == "ollama":
                result = await self.generate_ollama(request, request_id)
            else:
                result = await self.generate_hf(request, request_id, model)
            
            # Кешируем результат на 5 минут
            self.redis_client.setex(
                cache_key,
                300,
                result.json()
            )
            
            return result
            
        except Exception as e:
            logger.error(f"Generation failed for request {request_id}: {e}")
            raise HTTPException(status_code=500, detail=f"Generation failed: {e}")
    
    async def generate_ollama(self, request: GenerateRequest, request_id: str) -> GenerateResponse:
        """Генерирует текст с помощью Ollama"""
        start_time = time.time()
        
        try:
            response = self.ollama_client.generate(
                model=request.model,
                prompt=request.prompt,
                options={
                    'temperature': request.temperature,
                    'top_p': request.top_p,
                    'num_predict': request.max_tokens,
                    'stop': request.stop_sequences or []
                }
            )
            
            processing_time = int((time.time() - start_time) * 1000)
            
            return GenerateResponse(
                text=response['response'],
                tokens_used=response.get('eval_count', 0),
                processing_time_ms=processing_time,
                model_name=request.model,
                request_id=request_id
            )
            
        except Exception as e:
            logger.error(f"Ollama generation failed: {e}")
            raise
    
    async def generate_hf(self, request: GenerateRequest, request_id: str, model) -> GenerateResponse:
        """Генерирует текст с помощью HuggingFace модели"""
        start_time = time.time()
        
        try:
            tokenizer = self.tokenizers[request.model]
            
            # Готовим промпт
            prompt = request.prompt
            if request.system_prompt:
                prompt = f"{request.system_prompt}\n\n{prompt}"
            
            # Токенизируем
            inputs = tokenizer(
                prompt,
                return_tensors="pt",
                truncation=True,
                max_length=2048
            ).to(model.device)
            
            # Генерируем
            with torch.no_grad():
                outputs = model.generate(
                    inputs.input_ids,
                    max_new_tokens=request.max_tokens,
                    temperature=request.temperature,
                    top_p=request.top_p,
                    do_sample=True,
                    pad_token_id=tokenizer.eos_token_id,
                    eos_token_id=tokenizer.eos_token_id,
                    repetition_penalty=1.1
                )
            
            # Декодируем
            generated_text = tokenizer.decode(
                outputs[0][inputs.input_ids.shape[1]:],
                skip_special_tokens=True
            )
            
            # Применяем стоп-последовательности
            if request.stop_sequences:
                for stop_seq in request.stop_sequences:
                    if stop_seq in generated_text:
                        generated_text = generated_text.split(stop_seq)[0]
                        break
            
            processing_time = int((time.time() - start_time) * 1000)
            tokens_used = outputs[0].shape[1] - inputs.input_ids.shape[1]
            
            return GenerateResponse(
                text=generated_text.strip(),
                tokens_used=tokens_used,
                processing_time_ms=processing_time,
                model_name=request.model,
                request_id=request_id
            )
            
        except Exception as e:
            logger.error(f"HuggingFace generation failed: {e}")
            raise
    
    async def list_models(self) -> List[ModelInfo]:
        """Возвращает список доступных моделей"""
        models = []
        
        # Ollama модели
        try:
            ollama_models = self.ollama_client.list()
            for model in ollama_models['models']:
                models.append(ModelInfo(
                    name=model['name'],
                    type="ollama",
                    size=model.get('size', 'Unknown'),
                    description=f"Ollama model: {model['name']}",
                    created_at=datetime.fromisoformat(model['modified_at'].replace('Z', '+00:00')),
                    is_available=True
                ))
        except Exception as e:
            logger.warning(f"Failed to list Ollama models: {e}")
        
        # PEFT модели
        peft_dir = config.MODELS_DIR / "llm"
        if peft_dir.exists():
            for model_dir in peft_dir.iterdir():
                if model_dir.is_dir() and (model_dir / "adapter_config.json").exists():
                    try:
                        stat = model_dir.stat()
                        models.append(ModelInfo(
                            name=model_dir.name,
                            type="peft",
                            size="Unknown",
                            description=f"Fine-tuned model: {model_dir.name}",
                            created_at=datetime.fromtimestamp(stat.st_ctime),
                            is_available=True
                        ))
                    except Exception as e:
                        logger.warning(f"Failed to process PEFT model {model_dir}: {e}")
        
        return models

# Глобальный менеджер моделей
model_manager = ModelManager()

@asynccontextmanager
async def lifespan(app: FastAPI):
    """Lifecycle события приложения"""
    # Startup
    logger.info("Starting Heaven LLM Service")
    
    # Загружаем модель по умолчанию
    try:
        await model_manager.load_model(config.DEFAULT_MODEL)
        logger.info(f"Default model {config.DEFAULT_MODEL} loaded successfully")
    except Exception as e:
        logger.warning(f"Failed to load default model: {e}")
    
    yield
    
    # Shutdown
    logger.info("Shutting down Heaven LLM Service")

# Создаем FastAPI приложение
app = FastAPI(
    title="Heaven AI Arena - LLM Service",
    description="Сервис для работы с LLM моделями в Heaven AI Arena",
    version="0.1.0",
    lifespan=lifespan
)

# Настраиваем CORS
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Эндпоинты
@app.get("/health")
async def health_check():
    """Проверка здоровья сервиса"""
    return {
        "status": "healthy",
        "timestamp": datetime.utcnow().isoformat(),
        "device": config.DEVICE,
        "models_loaded": len(model_manager.models)
    }

@app.post("/generate", response_model=GenerateResponse)
async def generate_text(
    request: GenerateRequest,
    background_tasks: BackgroundTasks,
    db: Session = Depends(get_db)
):
    """Генерирует текст с помощью LLM"""
    start_time = time.time()
    
    try:
        result = await model_manager.generate_text(request)
        
        # Логируем запрос в фоне
        background_tasks.add_task(
            log_request,
            db=db,
            model_name=request.model,
            prompt_length=len(request.prompt),
            tokens_generated=result.tokens_used,
            processing_time_ms=result.processing_time_ms,
            success=True
        )
        
        return result
        
    except Exception as e:
        processing_time = int((time.time() - start_time) * 1000)
        
        # Логируем ошибку
        background_tasks.add_task(
            log_request,
            db=db,
            model_name=request.model,
            prompt_length=len(request.prompt),
            tokens_generated=0,
            processing_time_ms=processing_time,
            success=False,
            error_message=str(e)
        )
        
        raise

@app.get("/models", response_model=List[ModelInfo])
async def list_models():
    """Возвращает список доступных моделей"""
    return await model_manager.list_models()

@app.post("/train", response_model=TrainingResponse)
async def start_training(
    request: TrainingRequest,
    background_tasks: BackgroundTasks,
    db: Session = Depends(get_db)
):
    """Запускает обучение модели"""
    task_id = str(uuid.uuid4())
    
    # Создаем задачу в БД
    task = TrainingTask(
        id=task_id,
        model_name=request.model_name,
        dataset_path=request.dataset_path,
        output_name=request.output_name,
        config=request.json()
    )
    
    db.add(task)
    db.commit()
    
    # Запускаем обучение в фоне
    background_tasks.add_task(run_training, task_id, request)
    
    return TrainingResponse(
        task_id=task_id,
        status="started",
        message="Training task started successfully"
    )

@app.get("/training/{task_id}")
async def get_training_status(task_id: str, db: Session = Depends(get_db)):
    """Возвращает статус обучения"""
    task = db.query(TrainingTask).filter(TrainingTask.id == task_id).first()
    
    if not task:
        raise HTTPException(status_code=404, detail="Training task not found")
    
    return {
        "task_id": task.id,
        "status": task.status,
        "progress": task.progress,
        "created_at": task.created_at,
        "started_at": task.started_at,
        "completed_at": task.completed_at,
        "error_message": task.error_message
    }

# Вспомогательные функции
async def log_request(
    db: Session,
    model_name: str,
    prompt_length: int,
    tokens_generated: int,
    processing_time_ms: int,
    success: bool,
    error_message: str = None
):
    """Логирует запрос в базу данных"""
    try:
        log_entry = RequestLog(
            model_name=model_name,
            prompt_length=prompt_length,
            tokens_generated=tokens_generated,
            processing_time_ms=processing_time_ms,
            success=success,
            error_message=error_message
        )
        
        db.add(log_entry)
        db.commit()
        
    except Exception as e:
        logger.error(f"Failed to log request: {e}")

async def run_training(task_id: str, request: TrainingRequest):
    """Запускает процесс обучения"""
    # Placeholder для реального обучения
    # В реальной реализации здесь будет код для fine-tuning с PEFT
    logger.info(f"Training task {task_id} started")
    
    # Симуляция обучения
    await asyncio.sleep(5)
    
    # Обновляем статус
    db = SessionLocal()
    try:
        task = db.query(TrainingTask).filter(TrainingTask.id == task_id).first()
        if task:
            task.status = "completed"
            task.completed_at = datetime.utcnow()
            task.progress = 100.0
            db.commit()
            
        logger.info(f"Training task {task_id} completed")
        
    except Exception as e:
        logger.error(f"Training task {task_id} failed: {e}")
        
        task = db.query(TrainingTask).filter(TrainingTask.id == task_id).first()
        if task:
            task.status = "failed"
            task.error_message = str(e)
            db.commit()
            
    finally:
        db.close()

if __name__ == "__main__":
    uvicorn.run(
        "main:app",
        host="0.0.0.0",
        port=8000,
        reload=True,
        log_level="info"
    ) 