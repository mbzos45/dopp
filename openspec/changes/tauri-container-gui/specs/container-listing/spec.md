## Overview

コンテナ一覧を取得し、UIで表示する最小限の情報を定義する。

## Listing Rules

- 取得対象はすべてのコンテナ (稼働中/停止中の両方) とする。
- 並び順はエンジンが返す順序を基本とし、UI側で任意に並べ替えても良い。

## Data Shape

UIに渡すコンテナ情報の最小スキーマは以下とする。

- `id`: コンテナID (フルID)
- `name`: 表示名 (先頭の/を除去)
- `image`: イメージ名
- `state`: `running` | `exited` | `paused` | `created` | `restarting` | `dead` のいずれか
- `status`: エンジンが返す状態文字列

## Error Handling

- 取得失敗時は、UIに簡潔なエラーメッセージを表示する。
- 失敗理由はログに残す (エンジン接続/権限などは [engine-connection spec](../engine-connection/spec.md) に従う)。
