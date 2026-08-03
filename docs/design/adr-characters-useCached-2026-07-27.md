# ADR: Characters 三段事件 → useCached 适配方案

**日期**: 2026-07-27
**上下文**: Sprint 2 W3, Characters 页三段事件(event-based)与 useCached(Promise-based)不兼容

## 问题

`useCached<T>` 接收 `loader: () => Promise<T>`,但 Characters 的完整角色数据加载走 Tauri events:
1. `tauriInvoke('load_character_background')` 启动后台解析
2. Rust 端通过 Tauri event bus 分阶段推送 `char:stage1`/`stage2`/`stage3`/`char:error`
3. 组件注册 `listen()` 回调处理每个事件

结果:角色全量数据不走 L1 缓存,全靠手动 localStorage 读写,失效链路断裂。

## 方案对比

### 方案 A: 扩展 useCache hook 支持 event-based loader

- 给 `UseCachedOptions` 加 `eventLoader` 字段,内部用事件驱动而非 Promise
- hook 要管理事件生命周期(注册/取消/超时)

优点:通用性,将来其他 event-based 场景直接复用
缺点:钩子复杂度上升,`listen()` 是 Tauri 特有 API,不适合放进通用 hook;
`listen()` 在 React 组件外使用需要额外处理 cleanup

### 方案 B: Event→Promise adapter (selected ✅)

在 characterStore 加 `loadFull(name, saveFolder)` 方法,内部将事件流包装成 Promise。
Characters 页直接 `useCached({ loader: () => characterStore.loadFull(name, saveFolder) })`。

优点:
- 不改 useCache 基础设施,改动局限在 `cache/characters.ts` + `pages/Characters.tsx`
- adapter 可独立测试(返回 Promise,不依赖 Tauri runtime)
- 组件内状态管理简化(charStatus 派生自 useCached 的 loading/data)
- 事件生命周期在 Promise 内自然管理(注册 → resolve/reject → cleanup)

缺点:
- 如果将来 5 个组件都需要事件→Promise,需要抽公共 adapter
- stage1 仍然需要组件层副作用(更新 chip filter 的 class cache)

## 决策

选 **方案 B (event→Promise adapter)**。理由:
1. 改动范围最小,风险最低
2. useCache 的通用性不因 Tauri 特有 API 受损
3. adapter 可以独立演进,将来抽公共层时接口不改

## 实现要点

```ts
// characterStore.loadFull
async loadFull(name: string, saveFolder: string): Promise<CharacterInfo> {
  return new Promise((resolve, reject) => {
    const d2sPath = `${saveFolder}\\${name}.d2s`
    const timeout = setTimeout(() => { ... }, 20_000)
    const unsubs: (() => void)[] = []

    // stage1: 更新 class cache (chip filter 需要)
    listen('char:stage1', (e) => { updateL2ClassCache(name, e.payload) })
      .then(u => unsubs.push(u))
    // stage3: resolve
    listen('char:stage3', (e) => { cleanup(); setFull(name, data); resolve(data) })
      .then(u => unsubs.push(u))
    // error: reject
    listen('char:error', (e) => { cleanup(); reject(e.payload) })
      .then(u => unsubs.push(u))

    tauriInvoke('load_character_background', { path: d2sPath })
  })
}
```

Characters 页使用:
```ts
const { data: character, loading: charLoading } = useCached({
  key: characterStore.fullKey(selectedChar ?? ''),
  loader: () => characterStore.loadFull(selectedChar!, saveFolder),
  enabled: !!selectedChar && !characterStore.getFull(selectedChar),
})
```
