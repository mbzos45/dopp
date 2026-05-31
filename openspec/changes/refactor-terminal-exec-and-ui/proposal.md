## Why

現在の実装はクロスプラットフォームの複雑さ（macOS の AppleScript 起動、Linux の多段フォールバックによるターミナル探索）が蓄積しており、`crossbeam-channel` を使用している。また実装中に以下の潜在バグが判明した：複数コンテナへの同時アクション完了時にイベントが消失して `loading_containers` が永続化するバグ、および exec で起動したターミナルプロセスのゾンビ化問題。

## What Changes

- **macOS**: AppleScript 経由のターミナル起動（iTerm2 / Terminal.app）を削除し、exec ボタンはコマンドをクリップボードにコピーするのみとする
- **Linux**: ターミナル探索を `$TERMINAL` → `xdg-terminal-exec` → `x-terminal-emulator` の 3 段のみに簡略化し、ハードコードされたフォールバック（`gnome-terminal`、`konsole`、`xterm`）を削除する。`xdg-terminal-exec` 起動時は `--title=<コンテナ名>` を渡す
- **アーキテクチャ**: `ui.rs` を `main.rs` に統合してモジュール境界を解消する
- **チャネル**: `crossbeam-channel` を `tokio::sync::mpsc::unbounded_channel` に置き換え、イベントのキュー保証と同時アクション競合バグを修正する
- **ローディング UX**: Docker アクション実行中の `"(loading)"` テキストラベルを `egui::widgets::Spinner` に置き換える
- **バグ修正**: Docker アクション実行時のログが表示されない問題を修正し、エラーエリアにログ出力を表示する
- **バグ修正**: `spawn_command` で `Child` を即 drop せず待機スレッドで `wait()` することでゾンビプロセスを防止する

## Capabilities

### New Capabilities
- `exec-terminal`: プラットフォームごとの exec 動作 — macOS はクリップボードのみ、Linux は `--title=` 対応の簡略化されたターミナル探索

### Modified Capabilities

## Impact

- `src/terminal.rs`: macOS の AppleScript 起動パスを削除。`launch_linux` を `--title=` 注入付きの 3 段フォールバックに書き直す。`spawn_command` にゾンビ防止の待機スレッドを追加する
- `src/ui.rs` → `src/main.rs`: UI モジュールを統合して `ui.rs` を削除する
- `src/docker.rs`: `crossbeam_channel::Sender` を `tokio::sync::mpsc::UnboundedSender<UiEvent>` に置き換え、イベントを個別送信する
- `Cargo.toml`: `crossbeam-channel` 依存を削除する
