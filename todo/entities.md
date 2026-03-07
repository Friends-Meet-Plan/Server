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
