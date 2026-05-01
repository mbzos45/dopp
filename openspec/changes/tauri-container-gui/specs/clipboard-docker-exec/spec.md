## Overview

ターミナル自動起動の代替として、対象コンテナに接続するための
`docker exec -it` コマンドをクリップボードへコピーする。

## Command Template

- 生成フォーマット: `docker exec -it <container_id> /bin/bash`
- `<container_id>` は一覧取得で得たフルIDを使用する。

## Notes

- コンテナが停止中でもコマンド生成は可能とする。
- `/bin/bash` が存在しない場合はユーザーが編集して利用する前提とする。
