# Axiom Factory

LeanのNatural Number Gameに着想を得たPCブラウザ向け証明ゲーム
プレイヤーはLeanコードを書かず、論理式cardとtactic cardを選んで証明を進める

- 対象範囲: 直観主義命題論理 → 古典命題論理 → 一階述語論理 → 等号 → 自然数 → 整数 → 有理数
- フォルダ構成: `web/`(Svelte UI), `logic/`(Rust証明エンジン)
- Lean風だがLean自体は使わない。証明エンジンはRustで独自実装する

## 技術スタック

- Web: Bun, TypeScript, Svelte, Vite, Tailwind CSS, shadcn-svelte, KaTeX
- Logic: Rust, wasm-bindgen, tsify
- 静的ファイルにビルドし、サーバーは使わない
- 言語・フレームワーク・ライブラリは、すべて最新のバージョンを使用する

## 実装

- 初期段階のため、必要なら大胆に作り直してよい
- SvelteはUIのみに専念する
- ゲームロジック含め、UI以外はすべてRustに寄せる。状態もRustで持つ

## コード

### Rust

- Svelteに公開するAPIは必要最小限にする
- private/publicを問わず、通常の関数には**日本語で**doc commentを書く
- ただし、impl fmt::Display の fmt など、トレイト実装で明示的に呼ばれない関数のdoc commentは不要
- `Vec::new()`や`HashMap::new()`は使用せず、それぞれ`vec![]`、`hashmap!()`の形式で記述すること。

### Svelte

- コードは簡潔に書く
- 型は可能な限り強く制約する
- サードパーティーライブラリは積極的に自由に使用してよい
- PCブラウザに最適化する。スマホ対応は不要
- UIはRust APIの薄いアダプタとして実装する
- ダークモードのみ
