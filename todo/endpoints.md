# FRIENDS — endpoints

База: `/api/v1`

## Auth
- `POST /auth/register`
  - создать аккаунт
  - body: `{ name, username, avatar_url?, bio?, password }`
- `POST /auth/login`
  - логин по username/password
  - body: `{ username, password }`
- `POST /auth/logout`
  - завершить сессию

## Users
- `GET /users/me`
  - получить свой профиль
- `PATCH /users/me`
  - обновить свой профиль
  - body: `{ name?, avatar_url?, bio? }`
- `GET /users/:id`
  - получить профиль пользователя по id
- `GET /users/search?username=...`
  - поиск по username (starts_with или contains)

## Friendships
- `GET /friends`
  - список друзей (status=accepted)
- `GET /friends/requests/incoming`
  - входящие запросы (status=pending)
- `GET /friends/requests/outgoing`
  - исходящие запросы (status=pending)
- `POST /friends/requests`
  - отправить запрос в друзья пользователю `user_id`
  - body: `{ user_id }`
- `POST /friends/requests/:id/accept`
  - принять входящий запрос (status=accepted)
- `POST /friends/requests/:id/decline`
  - отклонить входящий запрос (status=declined или удалить запись)
- `DELETE /friends/:id`
  - удалить из друзей (разорвать accepted связь)

## Availability
- `GET /availability?user_id=...&from=YYYY-MM-DD&to=YYYY-MM-DD`
  - получить статусы дат пользователя в диапазоне
- `PUT /availability/:date`
  - обновить статус даты (если разрешено продуктом)
  - body: `{ status }`

## Events
- `POST /events`
  - создать событие и приглашения
  - body: `{ date, title, description?, location?, is_group, participant_ids[], wish_place_id? }`
    - wish_place_id? обсуждаемо можно просто убрать
- `GET /events/:id`
  - получить событие по id
- `GET /events?scope=upcoming|past|invited&user_id=...`
  - список событий по фильтру
- `PATCH /events/:id`
  - обновить поля события
  - body: `{ title?, description?, location?, status? }`
- `POST /events/:id/cancel`
  - отменить событие

## Event Participants / Invitations
- `GET /events/:id/participants`
  - список участников и их статусов
- `POST /events/:id/accept`
  - принять приглашение текущим пользователем (участник -> accepted)
- `POST /events/:id/decline`
  - отклонить приглашение текущим пользователем (участник -> declined, событие -> cancelled)
    - событие -> cancelled тоже обсуждаемо просто так проще

## Event Memories
- `POST /events/:id/memories`
  - добавить фото/воспоминание
  - body: `{ photo_url, caption? }`
- `GET /events/:id/memories`
  - получить фото/воспоминания события

## Wish List
- `GET /wish-places?user_id=...&status=active|visited|archived`
  - получить места пользователя по статусу
- `POST /wish-places`
  - создать место в wish list
  - body: `{ title, description?, location?, link? }`
- `PATCH /wish-places/:id`
  - обновить место (только владелец)
  - body: `{ title?, description?, location?, link?, status? }`
- `POST /wish-places/:id/visit`
  - отметить место как visited
  - body: `{ event_id }`
- `DELETE /wish-places/:id`
  - удалить/архивировать место

## Notifications
- `GET /notifications`
  - список уведомлений пользователя
- `POST /notifications/:id/read`
  - отметить уведомление прочитанным
- `POST /notifications/read-all`
  - отметить все уведомления прочитанными
