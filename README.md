# 概要
Windowsでも動く簡易的なdu

# 使い方
大体duと同じ。
詳細は 現在のステータス を見てください。
durs -d5 -h --threshold=1000000000 ${path}

# 目的
+ Rust学習用
+ Windowsでdu相当のものが欲しかったため

危険なunwrapの外し方しか分からないので一旦これ、、

# 現在のステータス
* 4オプションにのみ対応
  * -h対応
  * -d対応
  * --threshold対応
  * 対象フォルダ指定対応
  * 学習用なのでこれ以上の追加実装の予定はない
* 正常系しか考えず雑にunwrap
  * Rust学習用なのでここはまともにしたい