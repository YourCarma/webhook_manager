# Webhook Manager Service

## Структуру проекта

```
.
├── config
│   └── development.toml        #Конфигурация сервиса
├── src                      # Директория файлов логики работы сервиса
│   ├── bin              
│   │   └── run_server.rs        # Entrypoint сервиса
│   ├── server                # Директория работы контроллеров (серверный слой)
│   │   ├── router      
│   │   │   ├── mod.rs
│   │   │   ├── models.rs
│   │   │   └── storage.rs  
│   │   ├── config.rs
│   │   ├── error.rs
│   │   ├── mod.rs      
│   │   └── swagger.rs
│   ├── storage              # Директория модуля хранилища
│   │   ├── redis                # Директория логики работы Redis
│   │   │   ├── config.rs
│   │   │   ├── error.rs
│   │   │   └── mod.rs
│   │   ├── error.rs
│   │   ├── mod.rs
│   │   └── models.rs  
│   ├── config.rs
│   ├── errors.rs
│   ├── lib.rs
│   └── logger.rs
├── cargo.toml
└── .env
```

Заполнить

## Основные требования

ПУСТОТА

## Установка

ПУСТОТА

## Запуск

ПУСТОТА

### Вручную

ПУСТОТА

### Docker-compose

ПУСТОТА
