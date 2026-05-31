## 1. 依存関係の整理

- [x] 1.1 `Cargo.toml` から `crossbeam-channel` を削除する
- [x] 1.2 `tokio` の依存に変更がないことを確認する（`sync` フィーチャーが有効か確認）

## 2. チャネルの置き換え（docker.rs）

- [x] 2.1 `docker.rs` の `use crossbeam_channel::Sender` を `use tokio::sync::watch` に変更する
- [x] 2.2 `DockerRunner` の `tx` フィールド型を `tokio::sync::watch::Sender<Vec<UiEvent>>` に変更する
- [x] 2.3 `DockerRunner::new` のシグネチャを `(tx: watch::Sender<Vec<UiEvent>>)` に変更する
- [x] 2.4 `spawn_refresh` / `spawn_action` 内の `tx.send(...)` を `let _ = tx.send(vec![...])` に変更する
- [x] 2.5 Docker アクション実行中のログが `UiEvent` 経由で UI に伝わるよう、エラー時のメッセージを `UiEvent::ContainerActionFinished` の `result` に含めることを確認する

## 3. UI モジュールの統合（ui.rs → main.rs）

- [x] 3.1 `src/ui.rs` の全内容を `src/main.rs` に移動する
- [x] 3.2 `main.rs` の `mod ui;` と `use ui::{...}` を削除する
- [x] 3.3 `src/ui.rs` ファイルを削除する
- [x] 3.4 `MyApp::new` 内のチャネル生成を `tokio::sync::watch::channel(vec![])` に変更する
- [x] 3.5 `events` フィールド型を `watch::Receiver<Vec<UiEvent>>` に変更する
- [x] 3.6 `drain_events` を `watch::Receiver::borrow_and_update()` を使ったロジックに書き直す

## 4. ローディング表示の改善

- [x] 4.1 `is_loading` 時の `ui.label(format!("{name} (loading)"))` を `ui.add(egui::Spinner::new())` + `ui.label(&name)` に変更する

## 5. macOS exec の簡略化（terminal.rs）

- [x] 5.1 `launch_macos` 関数を削除する
- [x] 5.2 `run_osascript` / `escape_applescript_string` ヘルパー関数を削除する
- [x] 5.3 `launch_exec_terminal` の `#[cfg(target_os = "macos")]` ブランチを削除する
- [x] 5.4 macOS では `launch_exec_terminal` を呼ばず `copy_to_clipboard` のみを実行するよう `ui` 側のロジックを変更する（`#[cfg(target_os = "macos")]` で分岐）

## 6. Linux ターミナル探索の簡略化（terminal.rs）

- [x] 6.1 `launch_linux` から `gnome-terminal`・`konsole`・`xterm` のフォールバックを削除する
- [x] 6.2 `xdg-terminal-exec` の引数に `--title <container-name>` を追加する（`launch_linux` にコンテナ名を渡す形で実装）
- [x] 6.3 探索順が `$TERMINAL` → `xdg-terminal-exec`（`--title` 付き）→ `x-terminal-emulator` になっていることを確認する

## 7. チャネルを watch → mpsc に置き換え（同時アクション競合バグ修正）

- [x] 7.1 `docker.rs` の `use tokio::sync::watch` を `use tokio::sync::mpsc` に変更する
- [x] 7.2 `DockerRunner` の `tx` フィールド型を `mpsc::UnboundedSender<UiEvent>` に変更する
- [x] 7.3 `DockerRunner::new` のシグネチャを `(tx: mpsc::UnboundedSender<UiEvent>)` に変更する
- [x] 7.4 `spawn_refresh` の `tx.send(vec![...])` を `let _ = tx.send(UiEvent::ContainersRefreshed(...))` に変更する
- [x] 7.5 `spawn_action` の一括 `tx.send(vec![...])` を 2 回の個別 `tx.send()` に変更する
- [x] 7.6 `UiEvent` から `#[derive(Clone)]` を削除する（バッチ送信が不要になるため）
- [x] 7.7 `main.rs` の `use tokio::sync::watch` を `use tokio::sync::mpsc` に変更する
- [x] 7.8 `events` フィールド型を `mpsc::UnboundedReceiver<UiEvent>` に変更する
- [x] 7.9 `MyApp::new` のチャネル生成を `mpsc::unbounded_channel()` に変更する
- [x] 7.10 `drain_events` を `while let Ok(event) = self.events.try_recv()` ループに書き直す

## 8. ゾンビプロセス修正（terminal.rs）

- [x] 8.1 `spawn_command` で `Child` を即 drop するのをやめ、待機スレッドを立てて `child.wait()` を呼ぶよう変更する

## 9. 動作確認

- [x] 9.1 `cargo build` がエラーなく通ることを確認する
- [x] 9.2 `cargo test` が全て通ることを確認する
- [ ] 9.3 Linux 環境でコンテナの start / stop / restart が動作し Spinner が表示されることを確認する
- [ ] 9.4 Linux 環境で exec ボタンがターミナルを起動することを確認する
