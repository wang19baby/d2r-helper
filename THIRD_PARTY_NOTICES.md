# Third-Party Notices

本项目（D2R Marketplace）的代码以 GPLv3 许可（见 [LICENSE](LICENSE)）发布。
以下第三方内容各自适用其原始许可证：

## 开源分支说明

本仓库**不包含任何《暗黑破坏神 II：重制版》游戏提取资产**（物品图标、角色立绘、任务图标等均已移除，
相关的提取/下载脚本亦不提供）。应用 UI 图标需自行准备；仓库保留代码层面的路径约定
（`web/public/assets/img/items/{code}.png` 等），供使用者自行填充合规素材。

## 字体与图标字体

| 资产 | 位置 | 说明 |
|---|---|---|
| Font Awesome Free | `web/public/vendor/fa/` | 图标字体，适用 [Font Awesome Free License](https://fontawesome.com/license/free)（CC BY 4.0 + SIL OFL + MIT）。 |
| Google Fonts（IM Fell English SC / JetBrains Mono 等） | `web/public/fonts/` | 由 Google Fonts 提供，适用各自 OFL 许可（见各字体元数据）。 |

## 格式逆向参考（未包含在仓库内）

本项目实现的 `.d2s` / `.d2i` 位流格式解析，参考了以下开源社区成果（均在仓库外，按需自行获取）：

| 项目 | 作者 | 用途 |
|---|---|---|
| [D2SLib](https://github.com/dschu012/D2SLib) | dschu012 | d2s/d2i 读写参考（ItemStat 编码、header 布局） |
| [d2r-horadric-tools](https://github.com/crabsmadethis/d2r-horadric-tools) | crabsmadethis | d2s section marker 文档 |
| d2r-zero / construct_adapter | 社区 | Python 参考解析器（Rust 解析器与其逐页对拍） |
| D2R webpack chunk 3948 | — | 与 D2R.exe 同源的 Emscripten 解析代码，用于位序验证 |

## 测试样本

`src-tauri/tests/fixtures/` 中保留的 `.d2s` / `.d2i` 文件为格式解析测试样本，来源为社区生成或测试构造；
用户个人游戏存档不随仓库分发（见 `.gitignore`）。如样本涉及版权问题，请提 issue 联系移除。

## 商标

Diablo, Diablo II, Diablo II: Resurrected 是 Blizzard Entertainment, Inc. 的商标。本项目的使用仅为指代目的。
