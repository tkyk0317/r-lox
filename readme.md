# r-lox

以下の書籍をRustで実装

[インタプリタの作り方 －言語設計／開発の基本と2つの方式による実装－](https://www.amazon.co.jp/gp/product/4295017876/ref=ppx_yo_dt_b_asin_title_o01_s00?ie=UTF8&psc=1)

## コマンド

* make buld
  * ビルド実行
* make test
  * 単体テスト実行
* make run
  * sample/sample.loxを読み取り、tokenを表示
* make repl
  * REPL実行。入力した文字列をスキャンし、tokenを表示
* make act
  * github workflowsのシュミレート(actインストール必要)
