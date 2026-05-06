## Context

現在のUIはeguiの更新ループ内で`tokio::runtime::Runtime::block_on`を使ってDocker操作を呼び出しており、Docker APIの待ち時間でUIスレッドがブロックされています。アプリはeframe/eguiの単一プロセス構成で、`DockerClient`内にTokioランタイムを保持しています。

## Goals / Non-Goals

**Goals:**
- UIスレッドからすべてのブロッキング待機を排除する。
- Dockerの一覧/開始/停止/再起動をバックグラウンドタスクで実行し、結果をUIへ反映する。
- UI状態（コンテナ一覧、ロード中フラグ、エラー）を安全かつ一貫して更新する。
- 背景結果の到着時にUIを即時再描画する。

**Non-Goals:**
- ログストリーミングやexec機能の導入。
- 非同期化に不要なDocker APIの意味やタイムアウトの変更。
- UI状態の永続化。

## Decisions

- **非同期→UIの同期にメッセージチャンネルを採用する。**
  - `crossbeam_channel::Sender<UiEvent>`をタスク実行側が持ち、eguiアプリは`Receiver<UiEvent>`を保持する。
  - 理由: eguiの更新は単一スレッドで行われ、同期チャンネルを毎フレームdrainできる。`Arc<Mutex<...>>`のロック競合を避け、UIスレッドのみで状態更新を完結できる。
  - 代替案:
    - `Arc<Mutex<AppState>>`: 実装は簡単だが、描画中のロック競合や部分更新のリスクがある。
    - `tokio::sync::mpsc`: 非同期には適するが、UI側でのポーリングが煩雑になりやすい。

- **バックグラウンドタスクはTokioの`spawn`でFire-and-Forget。**
  - `spawn_refresh`/`spawn_start`/`spawn_stop`/`spawn_restart`のようなAPIで非同期処理を起動し、完了時に`UiEvent`を送る。
  - 理由: UIスレッドでの待機を完全に排除できる。

- **イベント到達時に明示的な再描画要求を行う。**
  - タスク起動時に`egui::Context`をクローンして保持し、`UiEvent`送信後に`request_repaint()`を呼ぶ。
  - 理由: ポーリングに頼らず、結果到達を即座に反映できる。

- **ロード中フラグはUIスレッドのみで更新する。**
  - タスク発行時に`is_loading`（またはアクション別フラグ）を立て、`UiEvent`処理時に解除する。
  - 理由: 共有状態の更新をUIスレッドに集約できる。

## Risks / Trade-offs

- **Risk**: `UiEvent`が未処理のままになる。 → **Mitigation**: 毎フレーム受信キューをdrainし、`request_repaint`を確実に呼ぶ。
- **Risk**: タスク実行中に`DockerClient`が破棄される。 → **Mitigation**: ランタイムをUIアプリが所有する長寿命のランナーに保持する。
- **Trade-off**: `crossbeam_channel`の追加依存。 → **Mitigation**: 小さく安定した依存で、用途は単一プロデューサに限定。

## Migration Plan

- `UiEvent`列挙型と`DockerTaskRunner`（または`DockerClient`拡張）を導入し、ランタイムと送信側を保持する。
- `MyApp`に`Receiver<UiEvent>`を持たせ、`ui()`内で毎フレームdrainする。
- 直接のDocker呼び出しをタスクディスパッチに置換し、`is_loading`を更新する。
- UI経路から`block_on`が消えていることを確認する（Docker APIは非同期タスク内のみ）。

## Open Questions

- グローバルな`is_loading`にするか、コンテナ単位のフラグにするか。
  コンテナ単位のフラグに

- start/stop/restart後に常に一覧更新するか、成功時のみ行うか。
  docker apiの応答が成功/失敗に関わらず状態が変わる可能性があるため、応答後に常に更新する
