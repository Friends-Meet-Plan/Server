# FRIENDS — сущности

## User
- id (uuid)
- username (string, unique)
- avatar_url (string, nullable)
- bio (string, nullable)

## Friendship
- id (uuid)
- user_id (uuid)
- friend_id (uuid)
- status (enum: pending, accepted)
- created_at

Замечания:
- направленная связь: инициатор = user_id, получатель = friend_id
- дружба считается принятой, когда status = accepted
- дружба уникальна по паре (user_id, friend_id)

## AvailabilityDay
- id (uuid)
- user_id (uuid)
- date (date)
- status (enum: invited, busy, past) если free то день не занят
- event_id (uuid, nullable)
- updated_at

Замечания:
- один день на пользователя
- status=invited при наличии входящего/исходящего приглашения
- status=busy при подтвержденной встрече
- status=past если date < today и есть event_id

## Event
- id (uuid)
- creator_id (uuid)
- date (date)
- title (string)
- description (string, nullable)
- location (string, nullable)
- is_group (bool)
- status (enum: pending, confirmed, cancelled, completed)
- wish_place_id (uuid, nullable)
- created_at

Замечания:
- участники события хранятся в EventParticipant (event_id, user_id)

## EventParticipant
- id (uuid)
- event_id (uuid)
- user_id (uuid)
- status (enum: pending, accepted, declined)

Замечания:
- уникально на пару (event_id, user_id)
- event.status = confirmed только когда все участники accepted
- при declined любого участника: event.status = cancelled (обсуждаемо)

// TODO: подумать
## EventMemory
- id (uuid)
- event_id (uuid)
- user_id (uuid)
- photo_url (string)
- visited_at

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
- visited_event_id (uuid, nullable)
- created_at
- updated_at

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
