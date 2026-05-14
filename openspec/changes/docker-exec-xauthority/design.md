# 設計: docker exec 起動時の XAUTHORITY 継承とクリップボードコピー

## 目的

`proposal.md` の仕様に基づき、`docker exec` コマンド生成ロジックを整理し、Linux 環境でホストの `XAUTHORITY` を必要に応じてコンテナへ渡す実装を行う。さらに、ターミナル起動成功時に生成コマンドをクリップボードへコピーする仕組みを導入する。

## 変更点（高レベル）

- コマンド組み立てロジックを独立した関数 `build_docker_exec_command(container_id: &str, extra_env: Option<&HashMap<String,String>>, config: &Config) -> String` に抽象化する。
- `XAUTHORITY` 継承判定はランタイムで行う（Linux のみ）。具体的には `cfg!(target_os = "linux")` と `std::env::var("XAUTHORITY")` の結果を組み合わせる。
- クリップボードコピーはプラットフォーム対応のライブラリ経由で行う（推奨: `copypasta` または `arboard`）。`docker.exec.copyCommandToClipboard` が `true` の場合のみコピーする。
- ユーザー設定はアプリ内の既存設定ストア（例: `Config` 構造体）に新しいフラグを追加する:
  - `copy_command_to_clipboard: bool` 既定: true
  - `inherit_xauthority_on_linux: bool` 既定: true

## 詳細設計

1. 設定読み取り
   - アプリ起動時または設定変更時に `Config` を読み込む。`inherit_xauthority_on_linux` と `copy_command_to_clipboard` を扱う。

2. コマンド生成ロジック
   - 既存の端末起動処理（`src/terminal.rs` の該当箇所）を修正し、シェル起動に使う `docker exec` を直接組み立てるのではなく `build_docker_exec_command` を呼ぶようにする。
   - `build_docker_exec_command` の振る舞い:
     - ベースコマンド: `docker exec -it <container> <shell>`
     - Linux 判定かつ `inherit_xauthority_on_linux == true` の場合:
       - `std::env::var("XAUTHORITY")` を取得する。
       - 値が存在しかつ非空なら `-e XAUTHORITY=$XAUTHORITY` を追加する。
     - それ以外は環境変数の追加は行わない。
     - `extra_env` 引数が渡された場合は、上記に加えそれらを `-e KEY=VALUE` 形式で追加する（必要に応じて既存の挙動を保持）。

3. クリップボード書き込み
   - `copy_command_to_clipboard == true` の場合、生成文字列をクリップボードに書き込む。
   - クリップボードライブラリは `arboard 3.6.1` を使用する。
   - 依存追加時は `default-features = false` を基本とし、必要最小限の feature のみを有効化する。
   - 追加する feature は実行環境に応じて最小構成に限定し、不要なバックエンドや拡張機能は有効化しない。
   - 失敗した場合はログ出力のみ行い、ユーザー操作を遮らない（フォールバック: 何もしない）。

4. セキュリティと注意点
   - `XAUTHORITY` の値は機密性がある可能性があるため、明示的なオプトアウト設定を用意する。
   - クリップボードへコピーすることも機密情報を共有する可能性があるため、ユーザーが無効化できるようにする。

5. テスト
   - ユニットテスト: `build_docker_exec_command` に対して以下を確認するテストを作成する。
     - Linux 想定で `XAUTHORITY` が設定されている場合、生成コマンドに `-e XAUTHORITY=` が含まれること。
     - `XAUTHORITY` が空または未定義の場合、生成コマンドに `-e XAUTHORITY` が含まれないこと。
     - 非 Linux 想定では `XAUTHORITY` が含まれないこと。
   - クリップボード操作はインテグレーションテストで検証（モック可能ならモックを使う）。

6. 依存関係
   - クリップボード機能のために `arboard 3.6.1` を最小 feature 構成で利用する。
   - 既存の設定ストアの API を拡張する。

## 変更対象ファイル（想定）

- `src/terminal.rs` — ターミナル起動フローの修正点。`build_docker_exec_command` を呼ぶ。
- `src/docker.rs` — もし既に docker コマンド生成ロジックがある場合はその位置に実装を統合。
- `src/ui.rs` — ユーザー設定 UI があればオプション追加。
- `Cargo.toml` — `arboard 3.6.1` を minimal features で追加。

## ロールアウト手順（開発→検証→本番）

1. ブランチ作成 `feat/docker-exec-xauthority`。
2. `build_docker_exec_command` 実装 + ユニットテスト追加。
3. クリップボード依存を追加し、動作確認。
4. 設定項目の UI 追加（必要なら）。
5. ローカル Linux 環境で E2E 検証（X アプリ起動確認）。
6. リリース。
