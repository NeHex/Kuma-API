# Kuma-API

一个使用 Rust 构建的简单 API 服务。

## 功能

- `GET /` 返回 JSON: `["hello,welcome to Kuma API; Visite: https://github.com/nehex/kuma-api"]`
- 服务端口：`7788`
- `GET /douban/{id}` 抓取 `https://movie.douban.com/subject/{id}`，返回封面、标题、年份、评分、简介和链接
- `GET /tmdb/{id}` 抓取 `https://www.themoviedb.org/movie/{id}`，返回封面、标题、年份、简介和链接

## Docker 部署

在项目目录执行：

```bash
docker compose up --build -d
```

验证接口：

```bash
curl http://127.0.0.1:7788/
```

预期返回：

```json
["hello,welcome to Kuma API; Visite: https://github.com/nehex/kuma-api"]
```

豆瓣接口示例：

```bash
curl http://127.0.0.1:7788/douban/1292052
```

示例返回：

```json
{
  "douban": {
    "cover": "https://img3.doubanio.com/view/photo/s_ratio_poster/public/p480747492.webp",
    "title": "肖申克的救赎",
    "years": "(1994)",
    "score": "9.7",
    "desc": "一场谋杀案使银行家安迪...",
    "url": "https://movie.douban.com/subject/1292052"
  }
}
```

TMDB 接口示例：

```bash
curl http://127.0.0.1:7788/tmdb/278
```

示例返回：

```json
{
  "tmdb": {
    "cover": "https://media.themoviedb.org/t/p/w220_and_h330_face/9cqNxx0GxF0bflZmeSMuL5tnGzr.jpg",
    "title": "The Shawshank Redemption",
    "years": "09/23/1994 (US)",
    "desc": "Imprisoned in the 1940s for the double murder of his wife and her lover...",
    "url": "https://www.themoviedb.org/movie/278"
  }
}
```

## 项目结构

```text
src/
  main.rs
  common/
    error.rs
    http.rs
    scraper.rs
  handlers/
    root.rs
    douban.rs
    tmdb.rs
```
