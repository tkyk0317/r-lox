# r-lox

以下の書籍をRustで実装

[インタプリタの作り方 －言語設計／開発の基本と2つの方式による実装－](https://www.amazon.co.jp/gp/product/4295017876/ref=ppx_yo_dt_b_asin_title_o01_s00?ie=UTF8&psc=1)

## 開発環境

* Mac Studio M2 Max/32GB
* Rust 1.72
* Docker 24.0.6

## コマンド

* make build
  * ビルド実行
* make test
  * 単体テスト実行
* make run
  * sample/sample.loxを読み取り、評価結果を表示
* make repl
  * REPL実行。入力した文字列をスキャンし、評価結果を表示
* make act
  * github workflowsのシュミレート(actインストール必要)

## 制限事項

* 以下の演算のみサポート
  * 四則演算
  * 等価演算子(==, !=, >, >=, <, <=)
  * 単項演算子(!=, -)
  * 数値、文字列、bool値、nil
  * 変数定義(var)、代入
  * print文
  * if、while、for、ブロック構文
* 文の末尾はセミコロンで終わる
* 数値
  * 全て浮動小数点として扱う
* 文字、文字列
  * ダブルクォーテーションで囲む
