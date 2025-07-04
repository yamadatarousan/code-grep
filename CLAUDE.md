# Claude開発ガイド - Code-Grep

## 🎯 プロジェクト概要
ripgrepを超える高速コード検索CLIツール（Rust製）- 現場で使える汎用開発ツール集の第2弾

## 🏗️ 技術スタック
- **言語**: Rust
- **検索エンジン**: 独自実装 + 正規表現最適化
- **CLI**: clap（引数解析）
- **並行処理**: rayon（マルチスレッド検索）
- **ファイル監視**: ignore（.gitignore対応）
- **出力**: 色付きハイライト、フォーマット対応

## 📋 開発ルール

### コマンド実行ルール
```bash
# ビルド・実行
cargo build
cargo run
cargo run -- --help
cargo run -- "検索パターン" [ディレクトリ]

# テスト
cargo test

# コード品質
cargo fmt        # フォーマット
cargo clippy     # リント

# リリースビルド
cargo build --release
```

### コード規約
1. **Rust**: 標準規約準拠、`cargo fmt`で自動フォーマット
2. **ファイル命名**: snake_case
3. **関数・変数**: snake_case
4. **構造体・列挙型**: PascalCase
5. **定数**: UPPER_SNAKE_CASE
6. **エラーハンドリング**: `Result<T, E>`必須

### Git運用
- コミット前に必ず`cargo test`実行
- コミットメッセージは日本語でOK
- featureブランチで開発、mainにマージ

## 📁 プロジェクト構造
```
code-grep/
├── src/
│   ├── main.rs           # CLIエントリーポイント
│   ├── lib.rs           # ライブラリルート
│   ├── searcher.rs      # コア検索エンジン
│   ├── matcher.rs       # パターンマッチング
│   ├── walker.rs        # ファイル走査
│   ├── output.rs        # 結果表示・フォーマット
│   ├── config.rs        # 設定管理
│   └── cli.rs           # CLI引数解析
├── Cargo.toml           # 依存関係・設定
├── tests/               # 統合テスト
├── benches/             # ベンチマーク
└── CLAUDE.md           # このファイル
```

## 🚀 開発優先順位

### Phase 1: 基本検索機能
1. ✅ プロジェクト初期化・CLAUDE.md作成
2. 🔄 Cargo.toml設定・依存関係追加
3. 🔄 CLI引数解析（clap使用）
4. 🔄 基本ファイル走査機能
5. 🔄 シンプルな文字列検索

### Phase 2: 高速化・高機能化
1. 🔄 正規表現検索
2. 🔄 マルチスレッド並行検索
3. 🔄 .gitignore/.codegreepignore対応
4. 🔄 ファイル拡張子フィルター
5. 🔄 バイナリファイル自動除外

### Phase 3: 出力・UX向上
1. 🔄 色付きハイライト出力
2. 🔄 行番号・コンテキスト表示
3. 🔄 JSON/CSV出力フォーマット
4. 🔄 統計情報表示
5. 🔄 設定ファイル対応

### Phase 4: 高度な機能
1. 🔄 構造化検索（関数内、クラス内等）
2. 🔄 言語固有の検索（Go, Rust, JS等）
3. 🔄 置換機能（--replace）
4. 🔄 インタラクティブモード
5. 🔄 検索履歴・ブックマーク

## 🎯 現在の開発状況
- ✅ プロジェクト初期化
- ✅ CLAUDE.md作成
- 🔄 Rustプロジェクト構造作成

## 🔧 使用例（予定）
```bash
# 基本検索
cg "function.*main"

# ディレクトリ指定
cg "TODO" ./src

# 拡張子フィルター
cg "import" --type rust

# 正規表現検索
cg --regex "fn\s+\w+\s*\(" --type rust

# 置換（プレビュー）
cg "old_function" --replace "new_function" --preview

# 実際の置換
cg "old_function" --replace "new_function" --write

# JSON出力
cg "error" --output json

# コンテキスト表示
cg "bug" --context 3

# 統計情報
cg "TODO" --stats

# 複数パターン検索
cg --patterns "TODO|FIXME|XXX"

# 除外パターン
cg "test" --ignore "*.log,*.tmp"
```

## 🎯 実装予定機能

### 基本機能
- [x] プロジェクト初期化
- [ ] CLI引数解析
- [ ] 基本文字列検索
- [ ] 正規表現検索
- [ ] ファイル走査・フィルタリング

### 高速化
- [ ] マルチスレッド検索
- [ ] メモリマップファイル
- [ ] SIMD最適化（可能であれば）
- [ ] ファイルタイプ別最適化

### UX・出力
- [ ] 色付きハイライト
- [ ] 行番号・コンテキスト表示
- [ ] プログレスバー
- [ ] JSON/CSV出力
- [ ] 統計情報

### 高度な機能
- [ ] 置換機能
- [ ] インタラクティブモード
- [ ] 言語固有検索
- [ ] 構造化検索
- [ ] 設定ファイル

## 🚨 ハマり検出・回避ルール

### 自動アラート条件
Claudeは以下を検出したら**必ず🚨アラートを出すこと**：

1. **同じファイルを4回以上連続編集**
2. **同じエラーパターンが3回以上出現**  
3. **ユーザーが「まだ」「また」「やっぱり」を使用**
4. **複数のデバッグアプローチを試しても15分以上解決しない**
5. **デバッグログを5個以上連続追加**

### アラート文言
「🚨 ハマり検出: 別のAI(GPT-4/Gemini)に相談するか、アプローチを変更しませんか？現在のアプローチを見直して、よりシンプルな解決策を探しましょう。」

### 強制実行ルール
- このルールは他の開発ルールと同等の優先度で**必ず実行する**
- アラート後は必ずユーザーに方針転換を提案する
- 「もっとシンプルな方法はありませんか？」を積極的に提案する

## 🏆 目標パフォーマンス
- **ripgrepと同等以上の速度**
- **大規模コードベース対応**（100万行以上）
- **低メモリ使用量**（<100MB）
- **起動時間**（<50ms）

## 🔧 設計思想（汎用ツール集共通）
- **軽量**: 依存関係最小、高速起動
- **クロスプラットフォーム**: Windows/macOS/Linux対応
- **自己完結**: 外部ツール依存なし
- **簡単インストール**: git clone → 一発セットアップ
- **直感的UI**: ヘルプ充実、使いやすいCLI

## 🔗 関連ツール
- **Everything**: ファイル名検索 ✅
- **code-grep**: コード検索 🔄
- **git-helper**: Git操作支援 📋
- **port-checker**: ポート管理 📋

## 🔧 次のタスク
1. Cargo.toml依存関係設定
2. 基本プロジェクト構造作成
3. CLI引数解析実装
4. 基本ファイル走査機能
5. シンプルな文字列検索実装