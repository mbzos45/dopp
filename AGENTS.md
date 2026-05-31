# dopp

## 実装時のルール

- Codeのコメントはすべて英語で作成
- ドキュメントファイルはすべて日本語で作成

## 修正時のコーディングルール

- 未実装部分を含む場合は、TODOコメントにすること。

## 修正時の確認

変更を行った際には、以下の点を確認してください。

1. `cargo clippy --all-targets --all-features -- -D warnings` がエラーなく通ること
2. `cargo fmt --all -- --check` がエラーなく通ること
3. `cargo test` が正常に動作すること
4. Tauriの場合、`cargo tauri build` が正常に動作すること