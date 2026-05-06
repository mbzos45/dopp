## Why

開発者が CLI コマンドや端末を行き来せずに Docker コンテナを操作できる、
軽量な GUI が必要です。用途を絞ったパネルにより、コンテキスト切替を減らし、
基本的な起動/停止の作業をより素早く行えます。

## What Changes

- コンテナ一覧 UI を追加し、各コンテナを1行で表示する。
- コンテナごとのアクション（start/stop/restart/exec）を提供し、
  状態に応じた表示制御を行う。
- 上部に close/refresh を追加し、refresh でコンテナ情報を再取得する。
- コンテナ数に合わせてウィンドウ高さを自動調整し、refresh で増えた場合は
  高さを延長する。
- exec は現時点では UI のみで、実処理は未実装とする。

## Capabilities

### New Capabilities
- `container-listing`: コンテナの状態と名前を行レイアウトで一覧表示する。
- `container-actions`: start/stop/restart/exec ボタンを状態に応じて表示する。
- `container-refresh`: コンテナ一覧を手動で再取得できる。
- `window-sizing`: コンテナ数に応じてウィンドウ高さを調整する。

### Modified Capabilities
- (なし)

## Impact

- UI レイヤー（`src/main.rs`）に egui/eframe のレイアウトとウィンドウ高さ調整を追加。
- Docker クライアント依存を追加/調整（使用バージョンの docs を参照し、非推奨 API は使用しない）。
- コンテナ一覧/refresh まわりのエラーハンドリングやログが必要になる可能性。
