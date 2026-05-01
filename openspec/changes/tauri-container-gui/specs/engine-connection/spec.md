## Overview

Docker/Podman互換のエンジンへ接続するための方針とエラーメッセージ仕様を定義する。
接続はTauriコマンド経由で実行し、UIには失敗理由を簡潔に伝える。

## Connection Priority

優先順位は以下の通りとする。

1. 環境変数 (`DOCKER_HOST`)
2. 既定ソケット
3. ユーザー設定 (将来拡張枠、現時点では未実装)

### Environment Variable

- `DOCKER_HOST` が設定されている場合はそれを採用する。
- `DOCKER_HOST` が無効な場合は接続失敗として扱い、次の候補には進まない。

### Default Socket Candidates

環境変数が無い場合、以下の候補を順に試す。

- `unix:///var/run/docker.sock`
- `unix:///run/podman/podman.sock`
- `unix://$XDG_RUNTIME_DIR/podman/podman.sock`
- `unix://$HOME/.podman/podman.sock`

### User Settings (Reserved)

- ユーザー設定は将来の拡張枠として予約する。
- 現時点ではUI/設定ファイルからの指定は行わない。

## Error Message Policy

UIで表示するエラーメッセージは以下のテンプレートを基準とする。

- 接続不可 (ソケット不在/未起動):
	- "エンジンに接続できません。Docker/Podmanが起動しているか確認してください。"
- 権限不足:
	- "エンジンにアクセスできません。ソケットの権限やグループ設定を確認してください。"
- タイムアウト/応答なし:
	- "エンジンの応答がありません。起動状態やネットワークを確認してください。"
- 不明なエラー:
	- "エンジン接続でエラーが発生しました。詳細はログを確認してください。"

## Logging Policy

Tauriの標準的なログ仕組みを利用し、以下の粒度で記録する。

- `info`: 接続試行開始/成功、接続先の種別(環境変数/既定ソケット)
- `warn`: リトライ不要の軽微な失敗(候補ソケット未存在など)
- `error`: 接続不能、権限不足、予期しない例外
