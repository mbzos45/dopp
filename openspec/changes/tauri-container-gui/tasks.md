## 1. Project Setup

- [x] 1.1 Bollard/anyhow/tauri関連の依存追加とCargo設定を整理する
- [x] 1.2 Tauriコマンドのスケルトンを追加する

## 2. Engine Connection

- [x] 2.1 Docker/Podman接続先の優先順位(環境変数 -> 既定ソケット -> ユーザー設定)を実装方針として整理する
- [x] 2.2 接続失敗時のエラーメッセージ仕様を決める

## 3. Container Listing

- [x] 3.1 コンテナ一覧取得のTauriコマンドを実装する
- [x] 3.2 フロントで一覧表示用の型/ストアを追加する

## 4. Container Lifecycle Controls

- [x] 4.1 起動/停止/再起動のTauriコマンドを実装する
- [x] 4.2 操作ボタンと実行状態のUIを追加する

## 5. Clipboard Docker Exec

- [x] 5.1 "docker exec -it <container_id> /bin/bash"の生成ロジックを定義する
- [x] 5.2 クリップボードコピーのTauri連携を追加する

## 6. Logging & Errors

- [x] 6.1 Tauriのログ仕組みを利用した出力方針を整理する
- [x] 6.2 主要操作のエラーをUIに通知できるようにする

## 7. UI Polish

- [x] 7.1 1行1コンテナのレイアウトを整える
- [x] 7.2 ウインドサイズ可変時の表示崩れを調整する

## 8. Validation

- [ ] 8.1 代表的な操作フローを手動で確認する
- [ ] 8.2 主要なエラーケース(未起動/権限不足)を確認する
