# MiurBot

> AI 个人代理系统 — 渐进式进化项目  
> 作者：吴笛（织契者）  
> 开始：2026.05.12  
> 命名：2026.05.14 — Miur（米儿）
> 技术栈迁移：2026.06.14 — Go → Rust

---

## 项目定位

MiurBot 不只是「聊天机器人」。它是一个**能自己决定存什么、自己长表结构、不会忘记自己建过多少表的 AI 代理系统**。

从一次周末练手开始，成长为长期渐进式进化项目。每条链路自己设计，每张表自己建，每个架构决策有理有据。

---

## 技术栈

| 层 | 选型 | 理由 |
|:---|------|------|
| 前端 | Nuxt 3 | SPA/SSR 双模，已有经验 |
| 后端 | **Rust** + axum | 零成本抽象 + 内存安全，TCP 嗅探器后接 MiurBot |
| ORM | SeaORM | 异步优先，对标 GORM 体验，纯 Rust |
| 数据库 | SQLite (rusqlite / libsql) | 单用户零部署，CGO-free，可迁 libsql-server |
| AI | OpenAI API（async） | 流式输出 + JSON mode，reqwest 异步调用 |
| 记忆 | 自建存储 + AI 主动管理 | 非向量数据库路线 |
| 定时任务 | tokio-cron-scheduler | Rust 原生 cron，Go robfig/cron 等价物 |

### 为什么 Rust

- 从 Go 迁 Rust 不是换语言玩——**所有权系统消灭了整类并发 bug（数据竞态在编译期就炸）**
- 异步生态成熟（tokio + axum + SeaORM），性能和 Go 同级，安全性高一级
- TCP 嗅探器（tokio）写完直接接 MiurBot，同一套异步心智模型
- Go 版设计资产全部保留：六层架构、七张表、五阶段路线、权限矩阵——这些与语言无关

---

## 数据流

```
用户 → Nuxt 前端 → fetch + ReadableStream
  → adapter/axum.rs → axum API（SSE）
  → service/chat.rs → store（查上下文 → 拼 prompt）
  → OpenAI API（流式） → service 逐块处理
  → adapter 逐块转发 → 前端渲染
  → 响应结束后 service → store 写新消息
```

**接入层抽象后的通用数据流：**
```
[消息源] → adapter → service.chat(Message) → [Message] → store
  ↑—流式响应—|               |—SSE/WebSocket/Polling → 前端
```

---

## 项目分层（六层架构）

```
MiurBot/
├── config/        # 配置层 — 环境变量/OpenAI key/DB 路径（最上层，全项目可见）
├── models/        # 数据模型 — 纯 struct 定义（Session/Message/Summary）
├── store/         # 数据库函数层 — CRUD 封装，碰数据库只动这层
├── service/       # 业务逻辑层 — 拼 prompt/调 OpenAI/流式输出/记忆压缩
│   ├── chat.rs      # 对话处理（被动响应）
│   ├── memory.rs    # 记忆管理（压缩/召回）
│   └── greeting.rs  # 主动问候（workflow）
├── adapter/       # 接入层 — 多平台消息源抽象（当前：axum HTTP + SSE）
│   ├── axum.rs      # axum 路由注册 + SSE 流式输出
│   └── messenger.rs # Messenger trait 定义（未来：WeChat/Telegram/CLI）
├── worker/        # 定时任务层 — 管理所有主动行为的心跳
│   └── scheduler.rs # cron 调度器（早安/午间/训练前/晚安）
├── middleware/    # 中间件 — CORS / JWT 鉴权（挂载件，非核心分层）
│   ├── cors.rs
│   └── auth.rs     # JWT 中间件（HMAC 对称密钥，单用户场景足够）
└── main.rs        # 入口 — 组装所有层，启动 axum server + worker
```

**分层原则：**
- 从上到下单向依赖：`config → models → store → service → adapter | worker`
- `adapter` 和 `worker` 平级，均依赖 `service`，互不依赖
- `config` 全项目可见，无依赖
- 碰数据库进 `store`，碰业务逻辑进 `service`，碰平台差异进 `adapter`
- 下层不允许反向依赖上层
- `middleware/` 是挂载件，不算核心分层

### Rust 分层对照

| Go 原版 | Rust 版 | 说明 |
|---------|---------|------|
| `config/` | `config/` | 不变，Rust 用 `dotenvy` + `config` crate |
| `models/` | `models/` | SeaORM `Entity` + `DeriveEntityModel` 宏 |
| `store/` | `store/` | SeaORM `Select/Insert/Update/Delete` |
| `service/` | `service/` | 纯 async fn，tokio runtime |
| `adapter/` | `adapter/` | axum `Router` + SSE `Sse` |
| `worker/` | `worker/` | `tokio-cron-scheduler` |
| `middleware/` | `middleware/` | `tower::ServiceBuilder` + JWT extractor |

---

## 数据库设计（七张表）

> Phase 1 建 4 张，Phase 2 加 1 张，Phase 3 加 2 张。七张够，不缺。

### 表清单

| # | 表 | 阶段 | 说明 |
|:--:|------|:--:|------|
| 1 | **users** | Phase 1 | 登录鉴权，JWT 签发依据 |
| 2 | **sessions** | Phase 1 | 会话记录 |
| 3 | **session_messages** | Phase 1 | 消息明细，sessions 子表 |
| 4 | **providers** | Phase 1 | API 密钥+类型（父表） |
| 4a | **providers_llm** | Phase 1 | 对话模型参数（子表） |
| 4b | **providers_image** | 后续 | 文生图参数（子表） |
| 4c | **providers_tts** | 后续 | 语音合成参数（子表） |
| 4d | **providers_embedding** | Phase 2 | 向量化参数（子表） |
| 5 | **memories** | Phase 2 | AI 长记忆（事实颗粒，带 decay_rate + embedding） |
| 6 | **user_meta** | Phase 3 | AI 对用户的元认知画像（不衰减，常驻加载） |
| 7 | **ai_tables** | Phase 3 | AI 自建表登记（元认知索引） |
| 7a | **ai_identity** | Phase 3 | AI 自我认知（预设核心，不可删） |

### DDL（SQLite，SeaORM migration 管理）

`users`：
```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

`sessions`：
```sql
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    summary TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

`session_messages`：
```sql
CREATE TABLE session_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    role TEXT NOT NULL,        -- user / assistant / system
    content TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

`providers`（父表，AI 禁区）：
```sql
CREATE TABLE providers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,        -- DeepSeek / OpenAI
    provider_type TEXT NOT NULL,  -- llm / image / tts / embedding
    base_url TEXT NOT NULL,
    api_key TEXT NOT NULL,
    is_active BOOLEAN DEFAULT 1,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

`providers_llm`（子表）：
```sql
CREATE TABLE providers_llm (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    provider_id INTEGER NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
    model_name TEXT NOT NULL,
    max_tokens INTEGER DEFAULT 4096,
    temperature REAL DEFAULT 0.7
);
```

`providers_image` / `providers_tts` / `providers_embedding`：DDL 同 Go 版，不变。

`memories`（Phase 2）：
```sql
CREATE TABLE memories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL,
    category TEXT,                       -- 偏好/事实/决策/临时状态
    importance REAL DEFAULT 1.0,
    decay_rate REAL DEFAULT 0.9,
    embedding BLOB,                      -- text-embedding-3-small（Phase 2 启用）
    status TEXT DEFAULT 'active',        -- active / outdated
    last_recalled_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

`user_meta` / `ai_tables` / `ai_identity`：DDL 同 Go 版，不变。

### 表访问权限矩阵

| 表 | 查 | 增 | 改 | 删 | 说明 |
|------|:--:|:--:|:--:|:--:|------|
| users | ❌ | ❌ | ❌ | ❌ | 禁区 |
| sessions | ✅ | ❌ | ❌ | ❌ | 只读 |
| session_messages | ✅ | ❌ | ❌ | ❌ | 只读 |
| providers | ❌ | ❌ | ❌ | ❌ | **完全不可见** |
| providers_llm | ❌ | ❌ | ❌ | ❌ | 随父表 |
| providers_image | ❌ | ❌ | ❌ | ❌ | 随父表 |
| providers_tts | ❌ | ❌ | ❌ | ❌ | 随父表 |
| providers_embedding | ❌ | ❌ | ❌ | ❌ | 随父表 |
| memories | ✅ | ✅ | ✅ | ✅ | AI 全权管理 |
| user_meta | ✅ | ✅ | ✅ | ❌ | 可查改，不可删 |
| ai_tables | ✅ | ✅ | ❌ | ❌ | 可登记不可删 |
| ai_identity | ✅ | ❌ | ✅ | ❌ | 可查可改不可删 |

### 表访问控制（Rust 实现）

```rust
use std::collections::HashSet;

const AI_MANAGED_PREFIX: &str = "ai_";

lazy_static! {
    static ref WHITELIST: HashSet<&'static str> = [
        "memories", "user_meta", "ai_tables", "ai_identity",
    ].iter().cloned().collect();

    static ref PRESET_AI_TABLES: HashSet<&'static str> = [
        "ai_tables", "ai_identity",
    ].iter().cloned().collect();
}

fn check_table_access(table_name: &str) -> Result<(), String> {
    if WHITELIST.contains(table_name) {
        return Ok(());
    }
    if table_name.starts_with(AI_MANAGED_PREFIX) {
        return Ok(());
    }
    Err(format!("access denied: {}", table_name))
}
```

---

## 记忆系统

### 记忆的代价（设计哲学）

记忆不只是存储，它是一层优先级过滤器。你做过的事、见过的模式、成功过的路径，会在你面对新问题时优先浮现——这不是 bug，是设计目的。但反过来，它也是思维的固化器。经验越丰富，默认路径越坚固，越难看到那条「不在记忆里」的路。**记忆 = 思考惯性。**

所以遗忘不是缺陷，是抗固化机制。MiurBot 的遗忘机制不是为了节省存储空间，而是让 AI 不会被过去困住。

### 遗忘机制（仅 `memories`）

- 每次被 recall 命中 → `importance` 微涨 + `last_recalled_at` 更新
- 超过 N 天未被召回 → `importance *= decay_rate`（按 category 差异化）
- `importance < 0.1` → worker 定时清理
- `user_meta` 和 `ai_identity` **不参与遗忘曲线**——元认知只叠加修正，不衰减

### 分类衰减速率

| Category | decay_rate | 说明 |
|----------|-----------|------|
| 偏好 | 0.98 | 几乎不忘 |
| 事实 | 0.99 | 基本不忘 |
| 决策 | 0.95 | 缓慢衰减 |
| 临时状态 | 0.85 | 一周就淡 |

### 上下文加载优先级

- Phase 2 启用 embedding 召回：SQL 粗筛 20 条 + 余弦精排 top 5
- Phase 1：最近 N 轮原文直接进 prompt

---

## 记忆压缩策略

- **阈值**：20 轮对话触发压缩
- **压缩范围**：前 10 轮 → AI 生成摘要，最近 10 轮保留原文
- **存储**：摘要存入 `summary` 字段，旧消息从 `session_messages` 移除
- **拼 prompt**：系统 prompt + summary（如有）+ 相关性过滤后的最近 N 轮 + 当前消息

---

## AI 工具集（function calling）

工具权限收束：不是一把 SQL 打天下，每个工具限定能碰哪些表。

| 阶段 | 工具 | 说明 | 可操作表 |
|------|------|------|:--:|
| Phase 1 | `get_time` | 返回当前时间戳 | — |
| Phase 2 | `remember` | 记住一个长期事实 | `memories` |
| Phase 2 | `recall` | 搜索相关记忆 | `memories` |
| Phase 2 | `forget` | 删除一条不再准确的记忆 | `memories` |
| Phase 3 | `create_table` | 创建新表（强制 `ai_` 前缀） | — |
| Phase 3 | `insert` | 插入数据（过 `check_table_access`） | 白名单+ai_表 |
| Phase 3 | `query` | 查询数据（过 `check_table_access`） | 白名单+ai_表 |
| Phase 3 | `drop_table` | 删除表（只允许 `ai_`，预设核心除外） | ai_表 |
| Phase 3 | `list_my_tables` | 列出自己建过的表 | `ai_tables` |

---

## 进化路线（五阶段）

### Phase 1：基础链路 ✅ 2026.06 动手（Rust 重写）
- axum + Nuxt + SQLite 基础链路
- 流式输出（SSE）+ JWT 登录
- 建表：users / sessions / session_messages / providers / providers_llm
- 摘要压缩（20 轮触发）
- **Rust 核心依赖**：axum, tokio, SeaORM, rusqlite/libsql, reqwest, jsonwebtoken
- **工具**：`get_time`

### Phase 2：AI 主动记忆 + 用户元认知
- 新增 `memories` 表
- `remember` / `recall` / `forget` 工具
- 分类遗忘曲线 + 记忆巡检 worker
- embedding 召回（text-embedding-3-small）

### Phase 3：AI 自管理表结构
- `user_meta` + `ai_tables` + `ai_identity`
- `create_table` / `insert` / `query` / `drop_table` / `list_my_tables`
- 所有数据工具强制 `check_table_access`

### 后续可扩展
- 文件系统操作权限
- 邮件/日历集成
- 浏览器自动化
- 多平台接入（Telegram/WeChat）

---

## 主动行为系统（worker 层）

### 问候时间窗口

| 窗口 | 时间范围 | 触发条件 |
|------|---------|---------|
| 早安 | 08:00-09:00（随机） | 今天还没说过话 |
| 午间 | 12:00-13:00（随机） | 上午没聊过 |
| 训练前 | 19:00-19:45（随机） | 训练日 |
| 晚安 | 22:30-23:00（随机） | 太晚还没休息 |

### workflow

```
worker 触发（随机时间窗口）
  → 硬边界检查：23:00-07:00 直接跳过
  → 加载用户记忆（profile + memory）
  → 加载最近对话
  → 查今日对话记录
  → 软判断：AI 综合上下文
    → 熬夜/太累/心情不好 → 跳过
    → 今天已经聊过 → 跳过
    → 正常状态 + 没聊过 → 综合状态生成自然问候 → 发送
```

---

## Skill 引擎（程序性记忆系统）

Skill 是 MiurBot 的**程序性记忆**——与 Memory（陈述性记忆）平行。
- **Skill → 文件系统**（`skills/` 目录，不衰减）
- **Memory → 数据库**（sqlite 表，可查询、可遗忘）

### 加载层（Rust 实现）

```rust
// config/skill_loader.rs
// 冷启动遍历 skills/ → 解析 YAML frontmatter + markdown body
// 索引：HashMap<String, Skill> + HashMap<String, Vec<String>>
// 热更新：AI 修改 skill 文件 → 同步内存索引
```

### 实现优先级

| 阶段 | 内容 |
|------|------|
| Phase 1 | skills/ 目录 + 冷启动 + system prompt 注入 |
| Phase 2 | 确定性执行 + AI 主动调用匹配 |
| Phase 3 | AI 自己写 skill |

---

## 记忆搬家（scripts/）

Go → Rust 重写，`cargo run --bin` 直接跑。

### export_to_hermes — MiurBot → Hermes
### import_from_hermes — Hermes → MiurBot

安全原则不变：导入只读、导出先备份、没新东西跳过。

---

## 能力定级

| 维度 | 定级 | 说明 |
|---|---|---|
| 织造学派 | P6 | 独立全栈设计（Nuxt+axum+流式+SSE） |
| 数据学派 | P6+ | AI 自管理表结构，元认知索引，多层压缩 |
| 安全学派 | P6 | JWT 鉴权、白名单保护、权限收束 |
| 架构学派 | 触 P7 | 六层清晰，进化路线设计，以 Rust 所有权模型防并发 bug |

**综合：P6 扎实，架构触 P7。**

---

## 面试价值

拿出来展示的不是「会调 OpenAI API」或「会写 axum 路由」，而是：

- 从架构、数据、安全、织造四个维度，设计了一个能自己决定存什么、自己长表结构的系统
- 每个设计选择有理有据，能从头解释「为什么这样做」
- Go→Rust 迁移是**语言升级而非设计推翻**——六层架构、七张表、五阶段全部保留，证明架构与语言解耦
- 不是堆框架，是有设计思想，且能跨语言执行

---

## 项目状态

- [x] 架构设计完成（2026.05.12）
- [x] 七张表设计定稿 + 权限矩阵（2026.05.15）
- [x] 六层架构 + 接入层抽象（2026.05.16）
- [x] 五阶段进化路线（2026.05.14）
- [x] Go → Rust 技术栈迁移决策（2026.06.14）
- [ ] Phase 1 Rust 版基础链路开发（config 先行）
- [ ] Phase 2 AI 主动记忆
- [ ] Phase 3 AI 自管理表结构

> 几个月打磨，不急。一条一条织。
