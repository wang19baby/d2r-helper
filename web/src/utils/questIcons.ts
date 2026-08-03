//! D2R 任务（Quest）图标 URL 工具
//!
//! 开源分支说明:quest 图标资源（原为 d2emu.com 下载的游戏提取图）
//! 已从仓库移除,不再随仓库分发。`questIconUrl` 统一返回 `null`,
//! 调用方已有 null 兜底（空槽位样式）。如需图标,自行提供
//! `web/src/assets/quest-icons/a{act}q{n}.png` 并恢复 `import.meta.glob`。

/**
 * 取出 (act, questId) 对应的图标 URL。
 *
 * 开源分支:恒返回 null（资源已移除）,调用方按空槽位渲染。
 *
 * @param act 1-5
 * @param questId 1-6 (act 内 0-based + 1,与 quest icon index 对齐)
 * @returns 图标 URL,或 `null` (开源分支恒 null)
 */
export function questIconUrl(act: number, questId: number): string | null {
  void act
  void questId
  return null
}

/**
 * 调试用:开源分支恒返回空列表。
 */
export function listQuestIcons(): string[] {
  return []
}
