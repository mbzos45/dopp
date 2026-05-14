# 実装タスク

- [x] 既存の `docker exec` 生成処理の所在を確認し、コマンド組み立て箇所を特定する
- [x] コマンド生成ロジックを関数化し、`XAUTHORITY` を条件付きで付与できるようにする
- [x] Linux 判定と `XAUTHORITY` の未設定時スキップ条件を実装する
- [x] `arboard 3.6.1` を `default-features = false` で追加し、最小 feature 構成でクリップボード操作を導入する
- [x] ターミナル起動成功時に生成コマンドをクリップボードへコピーする処理を追加する
- [x] 設定フラグ `copy_command_to_clipboard` と `inherit_xauthority_on_linux` を追加する
- [x] 設定が無効な場合の分岐を実装し、コピーと `XAUTHORITY` 継承を抑止する
- [x] コマンド生成ロジックのユニットテストを追加する
- [x] クリップボード処理の失敗時に起動処理へ影響しないことを確認する
- [ ] Linux 環境での手動確認手順を用意し、実機で検証する
