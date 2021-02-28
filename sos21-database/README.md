# `sos21-database`

[![docs (develop)](https://img.shields.io/badge/docs-develop-blue)](https://sohosai.github.io/sos21-backend/develop/sos21_database/)
[![docs](https://img.shields.io/github/v/release/sohosai/sos21-backend?label=docs&color=blue)](https://sohosai.github.io/sos21-backend/sos21_database/)

データベースとのインターフェースを提供します。

SQL を叩く関数群とデータモデルを実装しています。
`command` モジュールはデータを変更する操作、`query` モジュールはデータを変更しない操作を含んでいます。

## マイグレーション

`nix-shell` 内で次のコマンドを実行し、マイグレーションを適用します。

```shell
$ cd sos21-database
$ sqlx migrate run
```

## SQL のコンパイル時検証について

[`sqlx`](https://github.com/launchbadge/sqlx) クレートを用いて、
記述した SQL がある程度妥当なものであることをコンパイル時に検証しています。

`DATABASE_URL` 環境変数が設定されている場合は実際にデータベースに問い合わせることで検証を行い、
そうでない場合は `sqlx-data.json` に保存されている情報に基づいて検証を行います。

そのため、CI でビルドするためには `sqlx-data.json` を最新の状態に保つ必要があります。
`nix-shell` 内で次のコマンドを実行することで `sqlx-data.json` を更新することができます。

```shell
$ cd sos21-database
$ cargo sqlx prepare
```

詳しくは [Enable building in "offline" mode with `query!()`](https://github.com/launchbadge/sqlx/blob/master/sqlx-cli/README.md#enable-building-in-offline-mode-with-query) を参照してください。
