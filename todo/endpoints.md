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