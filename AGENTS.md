# Axiom Factory

LeanのNatural Number Gameに着想を得たPCブラウザ向け証明ゲーム
プレイヤーはLeanコードを書かず、論理式cardとtactic cardを選んで証明を進める

- 対象範囲: 直観主義命題論理 → 古典命題論理 → 一階述語論理 → 等号 → 自然数 → 整数 → 有理数
- フォルダ構成: `web/`(React UI), `logic/`(Rust証明エンジン)
- Lean風だがLean自体は使わない。証明エンジンはRustで独自実装する

## 技術スタック

- Web: TypeScript, React, React Compiler, Vite+, Tailwind CSS, shadcn/ui, KaTeX
- Logic: Rust, wasm-bindgen, tsify
- 静的ファイルにビルドし、サーバーは使わない
- 言語・フレームワーク・ライブラリは、すべて最新のバージョンを使用する

## 実装

- 初期段階のため、必要なら大胆に作り直してよい
- ReactはUIのみに専念する
- ゲームロジック含め、UI以外はすべてRustに寄せる。状態もRustで持つ

## コード

### Rust

- Reactに公開するAPIは必要最小限にする
- private/publicを問わず、通常の関数には**日本語で**doc commentを書く
- ただし、impl fmt::Display の fmt など、トレイト実装で明示的に呼ばれない関数のdoc commentは不要
- `Vec::new()`や`HashMap::new()`は使用せず、それぞれ`vec![]`、`hashmap!()`の形式で記述すること。

### React

- コードは簡潔に書く
- React Compiler前提で、不要なコードは避けること
- 型は可能な限り強く制約する
- サードパーティーライブラリは積極的に自由に使用してよい
- PCブラウザに最適化する。スマホ対応は不要
- UIはRust APIの薄いアダプタとして実装する
- ダークモードのみ

## UI

- 表示要素: current goal, goal navigation, tactic cards, theorem/definition inventory
- 操作: 論理式を選択 → 適用可能なtactic候補を表示 → tacticを選択 → 必要なら追加情報を入力 → 実行
- Tactic Card: 名前、説明、before/after preview、必要入力を表示
