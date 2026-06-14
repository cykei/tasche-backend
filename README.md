# Tasche Backend

Tasche 프로젝트의 백엔드 API 서버입니다.
Rust + Axum 기반이고, SQLite를 사용하며, 실행 시 마이그레이션을 자동 적용합니다.

## 요구사항

- Rust (1.93.0)
- Cargo (1.93.0)

## 환경설정

`.env` 파일에 `DATABASE_URL`을 설정할수 있습니다. 아직 다른 디비는 지원하지 않습니다. 

```env
DATABASE_URL=sqlite:data.db
```

## 실행

```bash
cargo run
```

서버는 기본적으로 `http://localhost:8080`에서 동작합니다.

## 데이터베이스

- SQLite 파일은 `data.db`로 생성됩니다.
- 애플리케이션 시작 시 `migrations/`가 자동 적용됩니다.

## API 엔드포인트 요약

- `GET /` Hello 메시지
- `GET /todos`
- `POST /todos`
- `PATCH /todos/:id`
- `DELETE /todos/:id`
- `DELETE /todos/:id/tags/:tag`
- `GET /tags`
- `DELETE /tags/:name`
- `GET /projects`
- `POST /projects`
- `DELETE /projects/:id`
- `GET /events`
- `POST /events`
- `DELETE /events/:id`
