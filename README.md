# Kuma-API

一个使用 Rust 构建的简单 API 服务。

## 功能

- `GET /` 返回 JSON: `["hello,welcome to Kuma API; Visite: https://github.com/nehex/kuma-api"]`
- 服务端口：`7788`
- `GET /douban/{id}` 抓取 `https://movie.douban.com/subject/{id}`，返回封面、标题、年份、评分、简介和链接
- `GET /tmdb/{id}` 抓取 `https://www.themoviedb.org/movie/{id}`，返回封面、标题、年份、简介和链接
- `GET /163music/{id}`：先请求 `https://node.api.xfabe.com/api/wangyi/music?id={id}` 获取歌曲直链，再请求 `https://api.paugram.com/netease/?id={id}` 获取歌曲信息，最后整合返回

## 中国大陆构建加速

项目已内置 Cargo 国内镜像配置：`./.cargo/config.toml`，默认使用 `rsproxy` 稀疏索引。

- 本机构建：

```bash
cargo build --release --locked
```

- 首次安装 Rust 工具链（可选，使用 USTC 镜像）：

```bash
export RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static
export RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup
rustup toolchain install stable
```

- Docker 构建：
  - `Dockerfile` 已优化为“先缓存依赖层，再编译业务代码”，二次构建速度明显更快。
  - 默认基础镜像走国内地址（`docker.m.daocloud.io`），避免直接拉取 `docker.io`。
  - 直接执行 `docker compose build` / `docker compose up --build -d` 即可。
  - 如需临时切换镜像源，可覆盖构建参数：

```bash
BUILDER_IMAGE=docker.m.daocloud.io/library/rust:1 \
RUNTIME_IMAGE=docker.m.daocloud.io/library/debian:bookworm-slim \
docker compose build
```

  - 如仍需加速 Docker Hub，可在宿主机配置 `registry-mirrors`（Linux）：

```bash
sudo mkdir -p /etc/docker
cat <<'EOF' | sudo tee /etc/docker/daemon.json
{
  "registry-mirrors": [
    "https://docker.m.daocloud.io",
    "https://hub-mirror.c.163.com"
  ]
}
EOF
sudo systemctl daemon-reload
sudo systemctl restart docker
docker info | grep -A2 "Registry Mirrors"
```

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

163 Music 接口示例：

```bash
curl http://127.0.0.1:7788/163music/33984984
```

示例返回结构：

```json
{
  "music163": {
    "stream_url": "https://m701.music.126.net/...mp3?...",
    "info": {
      "id": 33984984,
      "title": "穷孩子",
      "artist": "龍胆紫/PurpleSoul",
      "album": "W.T.F",
      "cover": "https://p1.music.126.net/...jpg?param=250y250",
      "lyric": "...",
      "sub_lyric": "",
      "link": "https://music.163.com/song/media/outer/url?id=33984984",
      "served": false,
      "cached": false,
      "remaining": 5
    }
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
    music163.rs
    root.rs
    douban.rs
    tmdb.rs
```
