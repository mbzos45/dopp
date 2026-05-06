## Why

UIスレッドがDocker APIの応答待ちでブロックされ、一覧取得やstart/stop操作中にフリーズが発生しています。Dockerの応答が遅い状況でもUIが常に応答できるよう、完全なノンブロッキング化が必要です。

## What Changes

- UIループおよびDocker操作から同期的な待機（`block_on`）をすべて排除する。
- 一覧/開始/停止を、UI再描画トリガー付きのFire-and-Forget非同期タスクに置き換える。
- 非同期タスクとeguiアプリ状態（コンテナ一覧、ロード中フラグ）を安全に同期する単一の共有/メッセージング層を導入する。
- 背景タスクからのエラー伝播とユーザー向けエラー表示更新を標準化する。

## Capabilities

### New Capabilities
- `async-docker-ops`: 背景タスク実行とUI状態同期を備えた、ノンブロッキングなDocker APIアクセス。

### Modified Capabilities
- 

## Impact

- `src/ui.rs`: UIイベントハンドラと更新ループ（非同期タスクのディスパッチ、再描画要求の追加）。
- `src/docker.rs`: Dockerクライアントの入口を非同期のみの形に整理。
- Runtime: Tokioのタスク起動と、状態共有用の同期プリミティブ（チャンネル or `Arc<Mutex<...>>`）。
