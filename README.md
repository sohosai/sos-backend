# 雙峰祭オンラインシステム バックエンド

[![CI](https://github.com/sohosai/sos21-backend/actions/workflows/ci.yml/badge.svg)](https://github.com/sohosai/sos21-backend/actions/workflows/ci.yml)
[![Release](https://github.com/sohosai/sos21-backend/actions/workflows/release.yml/badge.svg)](https://github.com/sohosai/sos21-backend/actions/workflows/release.yml)
[![Generate documentation](https://github.com/sohosai/sos21-backend/actions/workflows/generate-documentation.yml/badge.svg)](https://github.com/sohosai/sos21-backend/actions/workflows/generate-documentation.yml)
[![docs (develop)](https://img.shields.io/badge/docs-develop-blue)](https://sohosai.github.io/sos21-backend/develop/sos21_api_server/)
[![docs](https://img.shields.io/github/v/release/sohosai/sos21-backend?label=docs&color=blue)](https://sohosai.github.io/sos21-backend/sos21_api_server/)

- [API server documentation (develop)](https://sohosai.github.io/sos21-backend/develop/api-server.html)
- [API server documentation](https://sohosai.github.io/sos21-backend/api-server.html)

## Requirements

ローカルでの実行には以下のツールが必要です。

- [Docker](https://www.docker.com/)
- [Docker Compose](https://docs.docker.com/compose/)

これに加えて、開発には以下のツールが必要です。

- [Nix](https://nixos.org/nix/)
- [Cachix](https://cachix.org/)
  - あるとビルド時間が短くなります

## Run

次のコマンドでローカルに API サーバーを起動します。
これはリリースごとに push される `ghcr.io/sohosai/sos21-backend` イメージを用いています。

```shell
$ export SOS21_FIREBASE_PROJECT_ID=<project ID>
$ docker-compose -f docker-compose.run.yml up
```

`localhost:3000` で API サーバーが、
`localhost:4010` で [Prism](https://github.com/stoplightio/prism) の Validation Proxy を経由した API サーバーが利用できます。

## Development

ビルドの依存関係の固定に [Nix](https://nixos.org/nix/) を用いています。
[Cachix](https://cachix.org/) をインストールし、 `cachix use sos21-backend` するとバイナリキャッシュを利用できます。

```shell
$ cp .envrc.sample .envrc  # edit .envrc
$ source .envrc            # or use direnv
$ docker-compose -f docker-compose.dev.yml up -d
$ nix-shell
$ cargo run --bin sos21-api-server
```

[Prism](https://github.com/stoplightio/prism) の Validation Proxy が `localhost:4010` から利用できます。

### Migrations

`nix-shell` 内で次のコマンドを実行し、マイグレーションを適用します。

```shell
$ cd sos21-database
$ sqlx migrate run
```

詳しくは [`sos21-database`](sos21-database/README.md) を参照してください。
