# d2emu.com /hero 设计系统拆解

> 参考站点: <https://d2emu.com/hero> (D2R Save Editor & Hero Editor)
> 抓取时间: 2026-07-03
> 目标: 将该站点的设计语言迁移到 `扩展仓库` 与 `仓库` 页面,补强当前 D2 主题前端。

---

## 1. 一句话总结

一个**暗黑奇幻 + 影视级细节**的 Web UI:近黑底 + 烫金主色 + 血血红,大写 serif 标题,
6px 圆角卡片,虚线 drop 区,顶部条状 toast,带 D2 装备质感的渐变与发光。
**和当前项目 `web/src/index.css` 的 Cinzel + 血金主色方向高度一致**,只需做"加法":
对齐令牌、补 drop-zone / portrait / stepper / alert 三件套,即可在 `扩展仓`、`仓库` 中使用。

---

## 2. 颜色令牌 (推荐并入现有 `--color-d2-*`)

| 名称 | d2emu 值 | 当前值 | 用途 |
|---|---|---|---|
| `--d2s-bg` | `#0a0a0a` | `--color-d2-bg: #0f0d0b` | 页面底色 |
| `--d2s-panel` | `#111` | `--color-d2-panel: #1a1612` | 卡片底 |
| `--d2s-panel-2` | `#0e0e0e` | `--color-d2-panel2: #221d18` | 二级面板 |
| `--d2s-line` | `#252525` | `--color-d2-border: #3a2f25` | 主分隔线 |
| `--d2s-line-soft` | `#1a1a1a` | — | 软分隔线 |
| `--d2s-field` | `#1a1a1a` | — | 表单底 |
| `--d2s-text` | `#e8e8e8` | `--color-d2-text: #e7d9b8` | 正文 |
| `--d2s-muted` | `#888` | `--color-d2-text-soft: #b8aa88` | 次要文字 |
| `--d2s-gold` | `#FBB13A` | `--color-d2-gold: #c9a34a` | 烫金主色 |
| `--d2s-gold-bright` | `#ffffff` | — | 高亮金(几乎白) |
| `--d2s-link` | `#c7b377` | — | 链接 |
| `--d2s-red` | `#800000` | `--color-d2-btn: #7b2222` | 危险/主行动 |
| `--d2s-red-hover` | `#FBB13A` | — | hover 转金 |
| `--d2s-orange` | `#b87020` | `--color-d2-gold2: #8b6b2f` | 暖橙 |
| `--d2s-blue` | `#4f83c7` | — | 信息蓝 |
| `--d2s-bad` | `#ef5350` | — | 错误 |
| `--d2s-good` | `#4caf50` | — | 成功 |

> **不替换现有 `--color-d2-*`**。新增 `--d2emu-*` 命名空间(`web/src/index.css` `@theme` 内追加),
> 老页面继续走 D2 风,新页面 / 重做的扩展仓同时使用两套,避免颜色炸。

---

## 3. 字体 (建议补 Exocet + Source Sans 3)

| 角色 | d2emu | 现有 | 说明 |
|---|---|---|---|
| H1 / Hero Title | `Exocet` (暴雪授权感) + `Cinzel` fallback | `Cinzel` | 给"扩展仓库"做 hero 用 |
| Body | `Source Sans 3 300/400/600/700` | `Crimson Pro` | 无衬线易读 |
| Logo | `Roboto Mono` | — | 仅站点 logo |
| UI Text | `inherit` | `Crimson Pro` | 表单 |

**H1 排版规格**:
```css
font-family: 'Exocet', 'exocet blizzard ot', 'Cinzel', serif;
font-size: clamp(24px, 3.4vw, 38px);
font-weight: normal;
letter-spacing: 3px;
text-transform: uppercase;
text-align: center;
padding: 8px 20px;
```

字体文件来自 <https://d2emu.com/d2s/static/css/exocet-blizzard-light.ttf> (DMCA 风险,需替换)。
推荐替代:
- 站内已用 Cinzel 700/900 即可拿到 90% 效果
- 想要更接近: 免费替代 `Diablo`, `Cinzel Decorative`, `Cormorant SC`

---

## 4. 核心组件规范

### 4.1 卡片 (`.d2emu-card`)

```css
.d2emu-card {
  background: var(--d2emu-panel);
  border: 1px solid var(--d2emu-line);
  box-shadow: 0 2px 8px rgba(0,0,0,0.3);
  border-radius: 6px;
  padding: 14px;
}
```

适用: hero 区、统计块、物品详情面板。

### 4.2 Drop Zone (`.d2emu-drop`) ⭐

d2emu 的招牌:
```css
.d2emu-drop {
  min-height: 220px;
  border: 2px dashed var(--d2emu-gold);
  border-radius: 8px;
  background: linear-gradient(180deg, #1a1612, #110d08);
  box-shadow:
    0 4px 14px rgba(0,0,0,0.45),
    inset 0 0 30px rgba(154,0,0,0.06),
    0 0 0 1px rgba(154,0,0,0.18);
  animation: d2emu-drop-pulse 2.4s ease-in-out infinite;
}
@keyframes d2emu-drop-pulse {
  0%,100% { box-shadow: 0 4px 14px rgba(0,0,0,0.45), inset 0 0 30px rgba(154,0,0,0.06), 0 0 0 1px rgba(154,0,0,0.18), 0 0 0 0 rgba(154,0,0,0); }
  50%     { box-shadow: 0 4px 14px rgba(0,0,0,0.45), inset 0 0 30px rgba(154,0,0,0.10), 0 0 0 1px rgba(154,0,0,0.28), 0 0 24px 2px rgba(154,0,0,0.18); }
}
.d2emu-drop:hover { animation: none; border-color: #fff; background: linear-gradient(180deg,#221c14,#15110a); }
```

### 4.3 角色头像 (`.d2emu-portrait`)

```css
.d2emu-portrait {
  width: 58px; height: 58px; object-fit: cover;
  border: 1px solid var(--d2emu-gold); border-radius: 6px;
  background: radial-gradient(circle at 50% 15%, rgba(251,177,58,0.18), transparent 44%), #100a05;
  box-shadow: inset 0 0 10px rgba(0,0,0,0.55), 0 0 12px rgba(154,0,0,0.25);
}
```

扩展仓 / 仓库可以**用职业图标 / 角色卡片**做"我的赛季角色"入口。

### 4.4 数字步进器 (`.d2emu-stepper`)

```css
.d2emu-stepper { display: grid; grid-template-columns: 30px 1fr 30px; }
.d2emu-stepper button { background: #1f1f1f; color: #fff; border: 1px solid var(--d2emu-line); }
.d2emu-stepper button:hover:not(:disabled) { background: #262626; }
.d2emu-stepper button:first-child { border-radius: 4px 0 0 4px; border-right: 0; }
.d2emu-stepper button:last-child  { border-radius: 0 4px 4px 0; border-left: 0; }
.d2emu-stepper input { border-radius: 0; text-align: center; }
```

完美替代 `StashManager.tsx` 里那个简陋 `input type=number`。

### 4.5 字段对 (`.d2emu-field`)

```css
.d2emu-field { min-width: 0; }
.d2emu-field label { color: var(--d2emu-muted); font-size: 10px; text-transform: uppercase; letter-spacing: 0.08em; margin-bottom: 4px; display: block; }
.d2emu-field input, .d2emu-field select {
  width: 100%; box-sizing: border-box;
  padding: 7px 9px; border: 1px solid var(--d2emu-line);
  border-radius: 4px; background: var(--d2emu-field);
  color: var(--d2emu-text); font: inherit;
}
```

### 4.6 按钮 (`.d2emu-btn`)

```css
.d2emu-btn { padding: 10px 14px; border: 1px solid transparent; border-radius: 4px; cursor: pointer; font: inherit; font-weight: 600; transition: 120ms; }
.d2emu-btn-primary { background: transparent; border-color: #444; color: #ccc; }
.d2emu-btn-primary:hover { background: #1f1f1f; border-color: var(--d2emu-gold); color: #fff; }
.d2emu-btn-success { background: var(--d2emu-red); border-color: var(--d2emu-red); color: #fff; }
.d2emu-btn-success:hover { background: #a00000; }
.d2emu-btn-ghost    { background: transparent; border-color: var(--d2emu-line); color: var(--d2emu-text); }
.d2emu-btn-ghost:hover { border-color: var(--d2emu-gold); color: var(--d2emu-gold); }
```

> 比 `d2-btn`(血红渐变)更"档案级",适合现代深色档案风。
> 建议 `StashManager` 顶部操作行用 `.d2emu-btn-*` 替换。

### 4.7 顶部 Toast (`.d2emu-toast`)

当前 `components/Toast.tsx` 是底部条 + 简单圆角。d2emu 顶部居中、3 段配色:

```css
.d2emu-alert-host { position: fixed; top: 20px; left: 50%; transform: translateX(-50%); z-index: 10000; display: flex; flex-direction: column; gap: 8px; }
.d2emu-alert { min-width: 300px; max-width: 600px; padding: 12px 20px; border-radius: 6px; box-shadow: 0 6px 20px rgba(0,0,0,0.6); border: 1px solid; animation: d2emu-alert-in 0.25s ease-out; }
.d2emu-alert-info    { background: linear-gradient(180deg,#1a1a1a,#0e0e0e); border-color: var(--d2emu-gold); color: #fff; }
.d2emu-alert-success { background: linear-gradient(180deg,#0e2a18,#061806); border-color: var(--d2emu-good); color: #9ce5a8; }
.d2emu-alert-error   { background: linear-gradient(180deg,#2a0e0e,#180606); border-color: var(--d2emu-bad);  color: #ff9a9a; }
@keyframes d2emu-alert-in { from { opacity:0; transform: translateY(-12px); } to { opacity:1; transform: translateY(0); } }
```

### 4.8 Loading Panel (`.d2emu-loading`)

```css
.d2emu-loading { min-height: 320px; display: grid; place-items: center; border: 1px solid var(--d2emu-line); border-radius: 8px;
  background: radial-gradient(circle at 50% 42%, rgba(154,0,0,0.12), transparent 32%), linear-gradient(180deg, rgba(26,22,18,0.96), rgba(12,10,8,0.98));
  box-shadow: inset 0 0 42px rgba(0,0,0,0.55), 0 8px 24px rgba(0,0,0,0.35);
}
.d2emu-loading-mark { width: 96px; height: 96px; color: #fff; filter: drop-shadow(0 0 16px rgba(154,0,0,0.35)); }
.d2emu-loading-text { margin-top: 12px; font-size: 12px; font-weight: 700; letter-spacing: 0.18em; text-align: center; text-transform: uppercase; }
```

### 4.9 Tag 胶囊 (`.d2emu-tag`)

```css
.d2emu-tag { padding: 3px 10px; border: 1px solid var(--d2emu-line); border-radius: 999px; color: var(--d2emu-muted); background: transparent; font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.06em; }
```

---

## 5. 布局模式

### 5.1 主体三栏(主内容 + 粘性侧栏 + 底部)

```css
.d2emu-hero-layout { display: grid; grid-template-columns: minmax(0,1fr) minmax(250px,340px); gap: 12px; align-items: start; }
.d2emu-side-banner { position: sticky; top: 110px; max-width: 340px; }
@media (max-width: 899px) { .d2emu-side-banner { display: none; } }
```

### 5.2 角色主行(头像 + 名字 + 等级 + 职业 + 行动)

```css
.d2emu-character-main { display: grid; grid-template-columns: minmax(160px, 0.85fr) 130px minmax(140px, 0.6fr) auto; gap: 12px; align-items: end; }
```

### 5.3 4 列表格行(roster 风格)

```css
.d2emu-table { width: 100%; border-collapse: separate; border-spacing: 0; }
.d2emu-table thead th { padding: 10px 14px; background: #13171c; border-bottom: 2px solid var(--d2emu-gold); color: var(--d2emu-text); font-size: 0.85em; text-transform: uppercase; letter-spacing: 0.5px; }
.d2emu-table tbody tr:hover { background: rgba(74,158,255,0.06); }  /* 或 rgba(201,163,74,0.06) */
```

---

## 6. 动画

| 名称 | 时长 | 用途 |
|---|---|---|
| `d2emu-drop-pulse` | 2.4s ease-in-out infinite | drop-zone 呼吸光 |
| `d2emu-loading-spin` | 1.35s linear infinite | 加载环 |
| `d2emu-loading-pulse` | 1.6s ease-in-out infinite | 加载符文字 |
| `d2emu-alert-in/out` | 0.25s/0.2s | toast 滑入滑出 |
| `d2emu-rise` | 0.4s ease | 卡片入场上浮 |

---

## 7. 素材引用 (URL 全部抓自 d2emu.com/hero 真实请求)

### 字体
```
https://fonts.googleapis.com/css2?family=Cinzel:wght@600;700&display=swap
https://fonts.googleapis.com/css2?family=Source+Sans+3:wght@300;400;600;700&display=swap
https://d2emu.com/d2s/static/css/exocet-blizzard-light.ttf   ⚠ 仅作参考,不可直接分发
```

### 图标
```
https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.4.2/css/all.min.css
```
d2emu 实际使用的图标 (从 class 提取):
- `fa-cloud-arrow-up` (drop-zone)
- `fa-hat-wizard` (hero/角色)
- `fa-box-archive` (扩展仓)
- `fa-bookmark` (收藏/标签)
- `fa-folder-open` (载入)
- `fa-wand-magic-sparkles` (魔法/价格建议)
- `fa-plus`, `fa-xmark`, `fa-upload`, `fa-arrow-right`, `fa-bars`

### 站点图标 / Logo
```
https://d2emu.com/images/favicon.ico
https://d2emu.com/images/android-chrome-512x512.png
https://d2emu.com/d2s/static/img/hero-editor-thumb.png
```

> ⚠ d2emu 的图片资源是该站原创,我们项目**不直接复制**,继续用现有 `web/src/assets/img/items/*` 物品图。

### JS 框架
```
https://unpkg.com/react@18/umd/react.production.min.js
https://unpkg.com/react-dom@18/umd/react-dom.production.min.js
```
d2emu 走 React 18 UMD(纯前端) + jQuery 3.6.3; 我们项目走 React 19 + Vite + Tauri,无借鉴必要。

---

## 8. 落地路径 (在 `扩展仓` / `仓库` 中使用)

1. **`web/src/index.css`** — 在 `@theme {}` 内追加 `--color-d2emu-*` 令牌;
   文件末尾追加 `.d2emu-card / .d2emu-drop / .d2emu-portrait / .d2emu-stepper /
   .d2emu-field / .d2emu-btn-* / .d2emu-toast / .d2emu-loading / .d2emu-tag` 工具类。
2. **`web/src/pages/StashManager.tsx`**
   - 顶部标题块改用 `.d2emu-card` + Exocet 风 H1
   - 数字输入框改用 `.d2emu-stepper`
   - 顶栏 `↻ 刷新` 改用 `.d2emu-btn-ghost`
   - "存入扩展仓" CTA 改用 `.d2emu-btn-success` (血红实底)
   - 右侧扩展仓空状态用 `.d2emu-drop` 的"放入"提示版
3. **`web/src/pages/Inventory.tsx`**
   - 顶部统计卡改用 `.d2emu-card`
   - 物品页签改用 `.d2emu-tag` 风(更克制)
4. **`web/src/components/Toast.tsx`** — 升级支持 `.d2emu-toast` 三种 variant
   (顶部居中,可选定位)。向后兼容现有调用。
5. **新增 `web/src/components/D2EmuCard.tsx`** — 通用卡片壳,带可选
   `kicker` / `title` / `lede` / `tags` slot,直接复用 d2emu hero 段。

> **不在样式上替换现有 d2 风格** — `d2-*` 类继续负责 `Home/Catalog/Listings/Config`,
> `d2emu-*` 类负责 `Inventory/StashManager/Support` 偏档案级的新体验。
> 颜色令牌统一在 `@theme` 集中管理,后续可一键切换。

---

## 9. 风险与说明

- **Exocet 字体版权** — d2emu 自己也是 `@font-face` 直接引用,严格意义上属于
  "暴雪艺术资源未授权分发"。我们项目**不引入该 ttf**,用 Cinzel 700/900 替代效果。
- **fa-cloud-arrow-up 等图标** — FontAwesome 6.4.2 走 CDN 即可,与现有项目无冲突。
- **广告位 / 订阅** — d2emu 顶部有 NitoPay 视频广告和 `d2emu_subscriber` localStorage
  标记;我们 Tauri 桌面应用无广告场景,跳过。
- **响应式** — d2emu 在 899px 以下折叠侧栏,我们在桌面端默认不折叠,但保留
  `@media (max-width: 899px)` 的隐藏规则作为兼容。
