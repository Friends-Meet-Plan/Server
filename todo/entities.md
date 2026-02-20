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

Замечания:
- направленная связь: инициатор = user_id, получатель = friend_id
- дружба считается принятой, когда status = accepted
- дружба уникальна по паре (user_id, friend_id)

## Busyday

- id uuid pk
- user_id uuid not null
- date date not null
- event_id uuid null

unique(user_id, date)

## invitation
- id (uuid)
- from_user_id
- to_user_id
- status (pending, accepted, declined)
- selected_date (nullable, какая дата принята)
- created_at
ПРОВЕРКА НА friends

## invitation_dates (какие даты предложили)
- id
- invitation_id
- date

UNIQUE(invitation_id, date)

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

## EventMemory (для выполненых ивентов)
- id (uuid)
- event_id (uuid)
- user_id (uuid)
- надо еще id всех участников
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
