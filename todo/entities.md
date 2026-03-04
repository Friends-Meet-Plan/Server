## EventMemory (для выполненых ивентов)
- id (uuid)
- event_id (uuid)
- user_id (uuid) // тут все участники чтоб у каждого в проиле отоьразить
- photo_url (string)
- date

Замечания:
- 1 фото на пользователя на событие
- доступно после event.status = completed
- заполняет только пользователь

## WishPlace
- id (uuid)
- user_id (uuid)
- title (string)
- description (string, nullable)
- location (string, nullable)
- link (string, nullable)
- status (enum: active, visited, archived)
- created_at

Замечания:
- редактирует только владелец (обсуждаемо, для простоты можно убрать редакт)
- друзья могут использовать при создании встречи

## Notification через WebSocket (опционально для API) если mvp успеем
- id (uuid)
- user_id (uuid)
- type (enum: invite_new, invite_accepted, invite_declined, all_confirmed, event_tomorrow, event_add_photo, wish_visited)
- payload (json)
- is_read (bool)
- created_at
