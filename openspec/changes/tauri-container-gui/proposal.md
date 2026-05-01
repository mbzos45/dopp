## Why

ユーザーがコンテナの起動/停止などをシンプルに管理できるデスクトップGUIが不足しているため、ホスト環境向けに軽量でクロスプラットフォームな管理UIを提供する。
Docker/Podmanの両方を意識した最小限の操作に絞ることで、学習コストと保守コストを抑えつつ運用を簡易化する。

## What Changes

- Tauri + Svelteで、コンテナ一覧と操作ボタンを1行ずつ表示するシンプルなGUIを追加する。
- RustバックエンドでBollardを用いてDocker Engine API経由のコンテナ操作を行う。
- コンテナにアタッチする代替として、"docker exec -it"のコマンドをクリップボードにコピーできる。
- ロギングはTauriが提供する仕組み、またはTauriの依存に含まれる標準的なロギングを利用する。

## Capabilities

### New Capabilities
- `container-listing`: 既存コンテナを取得し、名前と状態を一覧表示する。
- `container-lifecycle-controls`: 起動/停止/再起動などの基本操作をUIから実行する。
- `clipboard-docker-exec`: 選択したコンテナ向けの"docker exec -it"コマンドをクリップボードにコピーする。
  ターミナル自動起動実装までのつなぎとして ペーストすれば目的のコンテナに接続できるようにする。
- `engine-connection`: Docker/Podman互換のエンジンへ接続し、ホスト側で操作できることを保証する。

### Modified Capabilities
- (none)

## Impact

- `src-tauri/`にBollardとanyhowを使ったコンテナ操作ロジックを追加する。
- `src/`のSvelte UIに一覧表示と操作ボタン、クリップボード連携を追加する。
- 依存関係にBollard関連のRustクレートが増える可能性がある。
- デスクトップアプリの権限/設定(Tauri)にDocker/Podman接続要件が影響する。
