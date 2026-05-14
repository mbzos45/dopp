# docker exec起動時の仕様変更

## 概要

ホストからコンテナへ端末で接続する際に利用する `docker exec` コマンドの動作を改善します。主に以下を実現します：

- ターミナル起動（コンテナ内のシェル開始）に成功した際、ユーザーが別ターミナルで同じ環境に簡単に接続できるよう、該当する `docker exec` コマンドをクリップボードに自動でコピーする機能を追加する。
- Linux 環境でホストの `XAUTHORITY` 環境変数が設定されている場合に限り、生成する `docker exec` コマンドへ `-e XAUTHORITY="$XAUTHORITY"` を付与してホストの X 認証情報をコンテナへ引き継ぐ（未設定時は上書きしない）。

## 背景・課題

- 複数のターミナルから同一コンテナに侵入したいユースケースがある。現状、ターミナルが起動した後に利用される `docker exec` コマンドが UI 上に明示されないため、ユーザーが手作業でコマンドを再構築する必要があるケースが生じる。
- 一部の最新 Linux ディストリビューションでは、`$XAUTHORITY` が再起動ごとにランダム化される挙動があり、Compose 等で作成したコンテナをホスト再起動後に GUI が利用できなくなる問題が観測されている。これを解決するため、ホストの `XAUTHORITY` を明示的にコンテナ側へ渡すことが有効である。

## 提案内容

1. ターミナル起動成功時に生成する `docker exec` コマンド文字列をクリップボードへ自動コピーする（ユーザー設定でオフ可能）。
  - 例: `docker exec -it -e XAUTHORITY=$XAUTHORITY <container> /bin/sh`
2. 上記で付与する `-e XAUTHORITY="$XAUTHORITY"` は **Linux のみ** の振る舞いとする。
3. ホスト側で `XAUTHORITY` が未設定（空や未定義）の場合は `-e XAUTHORITY=...` を一切付与しない（既存値の強制上書きは行わない）。
4. 動作は設定で有効／無効を切り替えられるようにし、デフォルトは安全策として有効（ただし GUI 関連のトラブルを避けたい場合は無効化可能）。

## 目的

- ユーザーの利便性向上：複数ターミナルでの同一コンテナ接続を容易にする。
- GUI 利用の信頼性向上：ホスト側の X 認証情報を必要に応じてコンテナへ引き継ぎ、再起動後の GUI 利用障害を軽減する。

## 仕様詳細

- 対象プラットフォーム：Linux のみで `XAUTHORITY` の自動継承を行う。クリップボードコピー機能はすべてのプラットフォームで利用可能とする。
- 環境変数の扱い：`XAUTHORITY` が空文字または未定義の場合は `-e XAUTHORITY=...` を付与しない。
- 生成される `docker exec` の例:

  - Linux（XAUTHORITY 設定あり）:

    `docker exec -it -e XAUTHORITY=$XAUTHORITY <container> /bin/sh`

  - Linux（XAUTHORITY 未設定）:

    `docker exec -it <container> /bin/sh`

  - 非 Linux（例：macOS/Windows）:

    `docker exec -it <container> /bin/sh`  （XAUTHORITY 自動追加なし）

- 設定項目（UI/CLI 両方で切り替え可能）:
  - `docker.exec.copyCommandToClipboard` (boolean) - デフォルト: true
  - `docker.exec.inheritXauthorityOnLinux` (boolean) - デフォルト: true

## 互換性・影響

- 既存の挙動は原則維持する（XAUTHORITY 未設定時は変更無し）。
- `XAUTHORITY` をコンテナに渡すことでホスト側の X 認証がコンテナ側で利用可能になり、GUI アプリケーションの起動安定性が向上する可能性が高い。
- セキュリティ面: ホストの X 認証ファイルをコンテナが参照するため、権限やアクセス制御に留意する。オプトアウト設定を必ず用意する。

## 実装案（高レベル）

1. 端末起動処理の箇所で `docker exec` コマンド文字列を組み立てるロジックを抽象化する。
2. ホストが Linux かどうかを判定し、かつ `process.env.XAUTHORITY`（またはランタイムでの環境取得）に値がある場合のみ `-e XAUTHORITY=...` を付与する。
3. 生成したコマンドをクリップボードへコピーする処理を呼ぶ（ライブラリ/ユーティリティを利用）。
4. ユーザー設定を読み取り、機能を有効/無効にするフラグを尊重する。

## 受け入れ基準

- ターミナル起動後、期待される `docker exec` コマンドが正しく生成され、クリップボードにコピーされること（設定有効時）。
- Linux で `XAUTHORITY` が設定されている場合、生成コマンドに `-e XAUTHORITY=...` が含まれること。
- ホスト側で `XAUTHORITY` が未設定の場合は生成コマンドに `-e XAUTHORITY` が含まれないこと。
- 設定を無効化した場合、クリップボードコピー・XAUTHORITY 継承の両方が行われないこと。

## ロールアウト

- 開発ブランチで実装し、手元の Linux 環境で動作検証を行う。
- ユーザーに影響が出ないようデフォルトで安全な挙動（既存要素の保持）を保ちつつ、設定で有効化する形で段階的に展開する。

---

Change directory: openspec/changes/docker-exec-xauthority/
