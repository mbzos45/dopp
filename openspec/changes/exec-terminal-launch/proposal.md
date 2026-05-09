## Why

execボタンが現在はno-opのため、コンテナ内部での調査や簡単な操作を行うたびにCLIへ切り替える必要があります。OS/ウインドマネージャー既定のターミナルを使って該当コンテナへ接続できるようにし、操作の手間とコンテキスト切替を減らします。

## What Changes

- execボタン押下で、対象コンテナに接続したターミナルを新規ウインドとして起動する。
- "[docker/podman] exec -it <コンテナ名/ID> /bin/bash" 形式のコマンドで接続する。
- OS別に既定ターミナルを優先して起動する。
  - Linux: $TERMINAL -> xdg-terminal-exec -> x-terminal-emulator、見つからなければ gnome-terminal -> konsole -> xterm。
  - Windows: startコマンドでOS設定に従う。失敗時は wt.exe -> cmd.exe を順に試す。
  - macOS: AppleScriptで iTerm2 -> Terminal.app の順に起動する。
- 起動失敗時はユーザーにエラー表示し、ログに詳細を残す。
  - 起動失敗時はdockerのコンソールに接続するコマンドを arboard v3.6.1を用いて クリップボードにコピーしてユーザーに手動での接続を促す。

## Capabilities

### New Capabilities
- `exec-terminal-launch`: execボタンで対象コンテナに接続したターミナルをOS既定の優先順位で起動する。

### Modified Capabilities
- (なし)

## Impact

- `src/ui.rs`: execボタンのクリック時にターミナル起動処理をトリガーする導線を追加。
- `src/docker.rs` または新規 `src/terminal.rs`: docker/podman execコマンド生成とOS別ターミナル起動のロジックを追加。
- 実行環境: macOSでのAppleScript実行、Windowsでのstart/wt/cmd呼び出し、Linuxでの端末解決のための外部コマンド呼び出し。
- エラーメッセージ表示とログ出力の拡充。
