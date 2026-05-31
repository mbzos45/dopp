## Context

`dopp` は egui/eframe ベースの Docker コンテナ管理 GUI アプリ。現在の実装は以下の課題を抱えている：

- `src/ui.rs` と `src/main.rs` がモジュール分割されているが、`ui.rs` は 246 行のみで分割の恩恵が薄く、参照が複雑になっている
- `crossbeam-channel` を非同期イベント伝達に使用しているが、外部クレートへの依存を避けるため tokio ネイティブのチャネルに統一したい
- Linux のターミナル起動フォールバックが 6 段（`$TERMINAL`、`xdg-terminal-exec`、`x-terminal-emulator`、`gnome-terminal`、`konsole`、`xterm`）と過剰で、保守性が低い
- macOS は AppleScript 経由で iTerm2 / Terminal.app を起動しようとするが、環境依存性が高くエラーになりやすい
- Docker アクション実行中のログが `tx.send` 後に `ctx.request_repaint()` が呼ばれる前に捨てられており UI に表示されない
- 複数コンテナへの同時アクション完了時にイベントが上書きされ `loading_containers` から ID が削除されないバグがある（watch の最新値のみ保持する特性による）
- exec で起動したターミナルプロセス（特に `xdg-terminal-exec` などの短命なラッパースクリプト）が `wait()` されずゾンビ化する

## Goals / Non-Goals

**Goals:**
- `ui.rs` を `main.rs` に統合してモジュール数を削減する
- チャネルを `crossbeam-channel` から `tokio::sync::mpsc::unbounded_channel` に置き換え、外部依存を削減しイベントのキュー保証を得る
- 同時アクション完了時のイベント消失バグ（`loading_containers` 永続化）を修正する
- macOS exec を clipboard-only に簡略化する
- Linux ターミナル探索を `$TERMINAL` → `xdg-terminal-exec`（`--title=` 付き） → `x-terminal-emulator` の 3 段に絞る
- ローディング表示を `Spinner` ウィジェットに置き換える
- Docker アクション結果ログを UI に表示されるよう修正する
- `spawn_command` のゾンビプロセス問題を待機スレッドで修正する

**Non-Goals:**
- Windows 対応の変更
- ターミナルエミュレータの追加・設定 UI
- Docker API 呼び出しロジックの変更

## Decisions

### 1. `ui.rs` を `main.rs` に統合

**決定**: `src/ui.rs` のすべての定義を `src/main.rs` に移動し、`mod ui;` 宣言と `ui.rs` ファイルを削除する。

**理由**: ファイルが 1 つになっても可読性は変わらない規模。モジュール境界を持つことで `pub` 可視性の管理コストが発生しているため、統合してシンプルにする。

### 2. `crossbeam-channel` → `tokio::sync::mpsc`

**決定**: `DockerRunner` 内の `Sender<UiEvent>` を `mpsc::UnboundedSender<UiEvent>` に置き換え、UI 側は `mpsc::UnboundedReceiver<UiEvent>` で `try_recv()` ループによりドレインする。

**理由**: `watch` は最新値のみ保持するため、複数コンテナへの同時アクション完了時にイベントが上書きされ `loading_containers` から ID が削除されないバグが発生する。`mpsc` はキューイング保証を持ちすべてのイベントを順序通りに届ける。`unbounded_channel` を選択した理由はイベントが人間操作の頻度でしか発生せずバックプレッシャーが不要なため。`try_recv()` は同期呼び出しなので egui の UI スレッドから直接使える。

**代替案**:
- `tokio::sync::watch` — 最新値上書き問題があり却下（→ `loading_containers` 永続化バグ）
- `Arc<Mutex<VecDeque<UiEvent>>>` — キュー保証はあるがボイラープレートが多い
- `std::sync::mpsc` — 追加依存なしだが tokio と混在させる必要がありコンテキストが複雑になる

### 3. macOS を clipboard-only に変更

**決定**: `launch_macos` 関数を削除し、macOS では `copy_to_clipboard` のみを実行する。

**理由**: AppleScript ベースの起動は iTerm2 / Terminal.app のインストール状況に依存し、エラーハンドリングが難しい。macOS ユーザーはターミナルアプリを自由に選択しており、クリップボードにコマンドを渡す方が汎用的。

### 4. Linux ターミナル探索の簡略化

**決定**: `$TERMINAL`（`-e` 付き）→ `xdg-terminal-exec`（`--title <name>` 付き）→ `x-terminal-emulator`（`-e` 付き）の 3 段に絞る。

**理由**: `xdg-terminal-exec` は XDG 標準に準拠した現代的な方法であり、`x-terminal-emulator` は Debian/Ubuntu 系で標準的な代替手段。`gnome-terminal`・`konsole`・`xterm` のハードコードは環境依存性が高く、`xdg-terminal-exec` / `x-terminal-emulator` が存在しない環境は現実的に少ない。`--title` はウィンドウタイトルにコンテナ名を表示してユーザビリティを向上させる。

### 5. ローディング表示を Spinner に変更

**決定**: `is_loading` 時の `"{name} (loading)"` テキストを `egui::Spinner::new()` ウィジェットに置き換える。

**理由**: Spinner はローディング状態を視覚的に分かりやすく伝える egui 標準ウィジェット。テキストより直感的。

### 6. ゾンビプロセス防止に待機スレッドを使用

**決定**: `spawn_command` で `Child` を即 drop するのをやめ、`std::thread::spawn` で待機スレッドを立てて `child.wait()` を呼ぶ。

**理由**: `xdg-terminal-exec` などのラッパースクリプトは Konsole 等を起動して即終了する短命プロセス。`Child` を `wait()` せずに drop すると、プロセス終了後も親（dopp）が生存している間ゾンビ状態になる。待機スレッドはターミナルが閉じるまで生存するが、1 回の exec につき 1 スレッドであり許容範囲。

**代替案**:
- `SIGCHLD` ハンドラ + `waitpid(-1, WNOHANG)` — より効率的だが `signal-hook` 等の依存追加が必要
- double-fork でプロセスを完全切り離し — コードが複雑になり過剰

## Risks / Trade-offs

- **mpsc unbounded のメモリ上限なし** → Docker イベントは人間操作頻度なので実害なし
- **macOS clipboard-only** → ターミナルが自動で開かなくなるため UX が変わる。ユーザーへの告知が必要な場合は UI にツールチップ等で補足する
- **ui.rs 統合** → `main.rs` が約 270 行になるが、この規模なら問題ない
- **待機スレッドの生存期間** → exec されたターミナルが開いている間スレッドが 1 本残る。多数の exec を行う環境でも OS スレッド数に対して無視できる量
