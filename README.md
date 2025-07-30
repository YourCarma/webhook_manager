# Webhook Manager Service

## Описание сервиса

  Сервис позволяет хранить процесс выполнения ресурсоемких задач пользователя посредством постоянного взаимодействия с целевым сервисом на основе**NoSQL-хранилища Redis.**

Основой взаимодействия с **WebhookManager** являются следущие структуры:
c
* **key** - структура на основе которой ведутся все **CRUD-операции** сервиса. Генерируется на стороне сервиса.
  <img src="./docs/key_content.png" alt="Структура ключа" style="display: block; margin-left: auto; margin-right: auto; width: 50%;"/>
* **task** - структура выполняемой задачи, имеющей как обязятельную стандартную структуру для всех сервисов часть, так и служебную информацию конкретного сервиса.
  <img src="./docs/task_content.png" alt="Структура ключа" style="display: block; margin-left: auto; margin-right: auto; width: 80%;"/>
  #### Описание структуры task:
    * `created_at` **(timestamp)** - время создания задачи;
    * `progress` **(float)** - прогресс задачи;
    * `status` **(str)** - статус выполнения задачи. Один из `[ PENDING, AWAITING, PROCESSING, READY, ERROR ]`;
    * `service` **(str)** - название сервиса, выполняющего задачу
    * `task_id` **(str)** - id выполняемой задачи
    * `user_id` **(str)** - id пользователя, запросивший задачу
    * `response_data` **(str(JSON) )** - служебная информация, зависящая от потребностей клиента

### Основные функции

#### 1) Получение задач по ключу
```rust
GET /storage/task?key={task_key}
```

#### 2) Создание задачи
```rust
POST /storage/task
```
```json
{
  "key": "guest:general:384f4d80-4ed6-4032-2569-f02fd5e1afb9",
  "task": {
    "created_at": "2025-07-09T12:51:27.948Z",
    "progress": {
      "progress": 0.1,
      "status": "PENDING"
    },
    "response_data": "string",
    "service": "general",
    "task_id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
    "updated_at": "2025-07-09T12:51:27.948Z",
    "user_id": "guest"
  }
}
```
#### 3) Удаление задачи
```rust
DELETE /storage/task?={task_key}
```

#### 4) Получение всех задач пользователя
Получение всех задач ведется по шаблону `{user_id}:*`, где `*` обозначает получение всех комбинаций задач пользователя
```rust
GET /storage/task?={user_id}:*
```

#### 5) Обновление прогресса задачи
```rust
POST /storage/update_progress
```
```json
{
  "key": "guest:general:384f4d80-4ed6-4032-2569-f02fd5e1afb9",
  "progress": {
    "progress": 0.1,
    "status": "PENDING"
  }
}
```

#### 6) Обновление служебной информации задачи
```rust
POST /storage/update_response_data
```
```json
{
  "key": "guest:general:384f4d80-4ed6-4032-2569-f02fd5e1afb9",
  "response_data": "JSON-string"
}
```

#### 7) Потоковое получение всех задач пользователя
```rust
WS /storage/ws
```
После подключения к вебсокету необходимо отправить сообщение шаблоном `{user_id}:*`, аналогично `GET` запросу **п.4**
```rust
ws.send("{user_id}:*")
```

#### 8) Получение метрик
```rust
GET /metrics
```

### Структура проекта

```yaml
.
├── config
│   └── development.toml        # Конфигурация сервиса
├── src                      	# Директория файлов логики работы сервиса
│   ├── bin  
│   │   └── run_server.rs       # Entrypoint сервиса
│   ├── server                  # Директория работы контроллеров (серверный слой)
│   │   ├── router  
│   │   │   ├── mod.rs
│   │   │   ├── models.rs
│   │   │   └── storage.rs  
│   │   ├── config.rs		# Конфигурация сервиса
│   │   ├── error.rs
│   │   ├── mod.rs  
│   │   └── swagger.rs
│   ├── storage              	# Директория модуля хранилища
│   │   ├── redis               # Директория логики работы Redis
│   │   │   ├── config.rs	# Конфигурация Redis
│   │   │   ├── error.rs	# Ошибки Redis
│   │   │   └── mod.rs
│   │   ├── config.rs		# Конфигурация хранилища
│   │   ├── error.rs		# Ошибки хранилища
│   │   ├── mod.rs
│   │   └── models.rs  		# Модели зачач
│   ├── config.rs		# Конфигурация сервиса
│   ├── errors.rs		# Ошибки сервиса
│   ├── lib.rs
│   └── logger.rs		# Конфигурация логгера
├── cargo.toml
└── .env
```
<img src="./docs/context.png" alt="Контекстная схема" style="display: block; margin-left: auto; margin-right: auto; width: 70%;" width="300"/>

## Основные требования

* Rust
* Redis
* Graphana (Optional) 
* Prometheus (Optional)

## Установка

ПУСТОТА

## Запуск

ПУСТОТА

### Вручную

ПУСТОТА

### Docker-compose

ПУСТОТА
