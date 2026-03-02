# FRIENDS — endpoints

База: `/api/v1`

## Auth
- `POST /auth/register`
  - создать аккаунт
  - body: `{ username, avatar_url?, bio?, password }`
- `POST /auth/login`
  - логин по username + password
  - body: `{ username, password }`
- `POST /auth/logout`
  - завершить сессию (клиент удалит токен)

## Users
- `GET /users/me`
  - получить свой профиль
- `PATCH /users/me`
  - обновить свой профиль
  - body: `{ username?, avatar_url?, bio? }`
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

## Calendar
- GET /users/{user_id}/calendar?from=2026-01-01&to=2026-04-30
Отдает дни для сетки (busy + можно сразу derived free на клиенте).
Лучше отдавать:
busy_days (даты + event_id),
pending_invites (даты pending invitation),
past_events (для отдельного цвета в прошлом).

## Events (новый основной flow)
- `POST /events`
Body: { date, title, description?, location?, wish_place_id?, participant_ids[] }
Логика:

creator = из auth
создаётся event
в user_events:
creator: role=owner, response_status=accepted
participants: role=participant, response_status=pending
проверки: только accepted-друзья, без self в participant_ids, без дублей

- `GET /events/{id}`
Возвращает событие + список участников из user_events

- `GET /events?scope=created|invited|upcoming|past`
Только для текущего пользователя

- `PATCH /events/{id}`
Body: { title?, description?, location? }
Только creator

- `POST /events/{id}/cancel`
Только creator, event.status -> canceled

синхронизация user_events/busydays

- `GET /events/{id}/participants`
Список из user_events (user_id, role, response_status)

- `POST /events/{id}/accept`
Только participant с pending
-> response_status=accepted
-> запись в busydays
-> если все accepted, event.status=confirmed

- `POST /events/{id}/decline`
Только participant с pending/accepted
-> response_status=declined
-> удалить его busyday для этого event_id (если был)
-> если осталось 2 человека (owner + 1) и второй declined, event.status=canceled; для group можно оставить pending/confirmed по твоему правил

---

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

ПОСЛЕ MVP
блокировать себе в профиле дни