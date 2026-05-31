## ADDED Requirements

### Requirement: macOS exec はクリップボードへのコピーのみ実行する
macOS 上で exec ボタンが押された場合、システムは AppleScript によるターミナル起動を行わず、exec コマンド文字列をクリップボードにコピーするのみとする。

#### Scenario: macOS で exec ボタンをクリック
- **WHEN** macOS 上でユーザーが実行中コンテナの exec ボタンをクリックする
- **THEN** exec コマンド文字列がクリップボードにコピーされる
- **THEN** ターミナルウィンドウは起動しない

---

### Requirement: Linux exec は $TERMINAL → xdg-terminal-exec → x-terminal-emulator の順で起動する
Linux 上で exec ボタンが押された場合、システムは以下の順序でターミナルを探索・起動する：
1. `$TERMINAL` 環境変数が設定されていれば `-e <cmd>` 付きで起動する
2. `xdg-terminal-exec` が存在すれば `--title <container-name>` と exec コマンドを引数として起動する
3. `x-terminal-emulator` が存在すれば `-e <cmd>` 付きで起動する
4. いずれも失敗した場合はエラーを返す

`gnome-terminal`・`konsole`・`xterm` へのフォールバックは行わない。

#### Scenario: $TERMINAL が設定されている
- **WHEN** `$TERMINAL` 環境変数に有効なターミナルパスが設定されている
- **THEN** そのターミナルが `-e <exec-cmd>` 付きで起動される

#### Scenario: $TERMINAL 未設定かつ xdg-terminal-exec が利用可能
- **WHEN** `$TERMINAL` が未設定で `xdg-terminal-exec` が存在する
- **THEN** `xdg-terminal-exec --title <container-name> <exec-cmd>` が実行される

#### Scenario: xdg-terminal-exec も不在で x-terminal-emulator が利用可能
- **WHEN** `$TERMINAL` が未設定かつ `xdg-terminal-exec` が存在せず `x-terminal-emulator` が存在する
- **THEN** `x-terminal-emulator -e <exec-cmd>` が実行される

#### Scenario: すべてのターミナルが起動失敗
- **WHEN** いずれのターミナルも起動できない
- **THEN** エラーが UI のエラーエリアに表示される
