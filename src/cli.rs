use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Clone)]
#[command(name = "cg")]
#[command(about = "🔍 高速コード検索CLIツール - ripgrepを超える", long_about = None)]
#[command(version, author)]
pub struct Cli {
    /// 検索パターン
    #[arg(value_name = "PATTERN")]
    pub pattern: Option<String>,

    /// 検索対象ディレクトリ/ファイル
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,

    /// 正規表現検索を有効にする
    #[arg(short, long)]
    pub regex: bool,

    /// ファンシー正規表現を使用（先読み・後読み対応）
    #[arg(long)]
    pub fancy_regex: bool,

    /// 大文字小文字を区別する
    #[arg(short = 'c', long)]
    pub case_sensitive: bool,

    /// 単語境界で検索
    #[arg(short, long)]
    pub word_boundary: bool,

    /// リテラル検索（正規表現無効）
    #[arg(short, long)]
    pub literal: bool,

    /// ファイルタイプで絞り込み
    #[arg(short, long, value_delimiter = ',')]
    pub r#type: Vec<String>,

    /// 拡張子で絞り込み
    #[arg(short, long, value_delimiter = ',')]
    pub ext: Vec<String>,

    /// 除外パターン
    #[arg(long, value_delimiter = ',')]
    pub ignore: Vec<String>,

    /// .gitignoreを尊重する
    #[arg(long, default_value = "true")]
    pub respect_gitignore: bool,

    /// 隠しファイルも検索
    #[arg(long)]
    pub hidden: bool,

    /// バイナリファイルも検索
    #[arg(long)]
    pub binary: bool,

    /// 最大ファイルサイズ（例: 1M, 500K）
    #[arg(long)]
    pub max_filesize: Option<String>,

    /// 最大検索深度
    #[arg(long)]
    pub max_depth: Option<usize>,

    /// 最近変更されたファイルのみ（例: 7d, 2h）
    #[arg(long)]
    pub modified_within: Option<String>,

    /// 置換文字列
    #[arg(long)]
    pub replace: Option<String>,

    /// プレビューモード（実際には変更しない）
    #[arg(long)]
    pub preview: bool,

    /// インタラクティブ置換
    #[arg(short = 'i', long)]
    pub interactive: bool,

    /// ファイルに書き込み
    #[arg(long)]
    pub write: bool,

    /// 行番号を表示
    #[arg(short = 'n', long)]
    pub line_numbers: bool,

    /// コンテキスト行数
    #[arg(short = 'C', long, default_value = "0")]
    pub context: usize,

    /// 前コンテキスト行数
    #[arg(short = 'B', long)]
    pub before_context: Option<usize>,

    /// 後コンテキスト行数
    #[arg(short = 'A', long)]
    pub after_context: Option<usize>,

    /// 出力フォーマット
    #[arg(short, long, default_value = "text")]
    pub output: OutputFormat,

    /// 色付き出力
    #[arg(long, default_value = "auto")]
    pub color: ColorChoice,

    /// ファイル名のみ表示
    #[arg(long)]
    pub files_only: bool,

    /// マッチ数のみ表示
    #[arg(long)]
    pub count_only: bool,

    /// 統計情報のみ表示
    #[arg(long)]
    pub stats_only: bool,

    /// 並行スレッド数
    #[arg(short = 'j', long)]
    pub threads: Option<usize>,

    /// 最大メモリ使用量（例: 512M, 1G）
    #[arg(long)]
    pub max_memory: Option<String>,

    /// 高速モード（精度より速度優先）
    #[arg(long)]
    pub fast: bool,

    /// 詳細モード（精度優先）
    #[arg(long)]
    pub thorough: bool,

    /// 関数内のみ検索
    #[arg(long)]
    pub functions: bool,

    /// 特定関数内のみ検索
    #[arg(long)]
    pub in_function: Option<String>,

    /// 特定クラス内のみ検索
    #[arg(long)]
    pub in_class: Option<String>,

    /// 特定スコープ内のみ検索
    #[arg(long, value_delimiter = ',')]
    pub in_scope: Vec<String>,

    /// インポート文のみ検索
    #[arg(long)]
    pub imports_only: bool,

    /// コメント内のみ検索
    #[arg(long)]
    pub comments_only: bool,

    /// 複数パターンのAND検索
    #[arg(long, value_delimiter = ',')]
    pub and: Vec<String>,

    /// 複数パターンのOR検索
    #[arg(long, value_delimiter = ',')]
    pub or: Vec<String>,

    /// パターンファイル
    #[arg(long)]
    pub pattern_file: Option<PathBuf>,

    /// 言語固有ルールファイル
    #[arg(long)]
    pub lang_rules: Option<PathBuf>,

    /// 出力フィルター
    #[arg(long)]
    pub output_filter: Option<PathBuf>,

    /// 設定ファイル
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// インタラクティブモード
    #[arg(long)]
    pub interactive_mode: bool,

    /// ファジー検索
    #[arg(long)]
    pub fuzzy: bool,

    /// リアルタイム検索
    #[arg(long)]
    pub live: bool,

    /// 正規表現チェック
    #[arg(long)]
    pub check_regex: Option<String>,

    /// サブコマンド
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Clone)]
pub enum Commands {
    /// 設定を表示
    Config {
        /// 設定ファイルパス
        #[arg(long)]
        path: Option<PathBuf>,
        /// デフォルト設定を出力
        #[arg(long)]
        default: bool,
    },
    /// ベンチマークを実行
    Benchmark {
        /// ベンチマーク対象パターン
        pattern: String,
        /// ベンチマーク対象ディレクトリ
        path: Option<PathBuf>,
        /// 繰り返し回数
        #[arg(short, long, default_value = "10")]
        iterations: usize,
    },
    /// 言語固有のヘルプを表示
    LangHelp {
        /// 言語名
        language: Option<String>,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    /// プレーンテキスト
    Text,
    /// JSON形式
    Json,
    /// CSV形式
    Csv,
    /// XML形式
    Xml,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ColorChoice {
    /// 自動判定
    Auto,
    /// 常に色付き
    Always,
    /// 色なし
    Never,
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            pattern: None,
            paths: vec![],
            regex: false,
            fancy_regex: false,
            case_sensitive: false,
            word_boundary: false,
            literal: false,
            r#type: vec![],
            ext: vec![],
            ignore: vec![],
            respect_gitignore: true,
            hidden: false,
            binary: false,
            max_filesize: None,
            max_depth: None,
            modified_within: None,
            replace: None,
            preview: false,
            interactive: false,
            write: false,
            line_numbers: false,
            context: 0,
            before_context: None,
            after_context: None,
            output: OutputFormat::Text,
            color: ColorChoice::Auto,
            files_only: false,
            count_only: false,
            stats_only: false,
            threads: None,
            max_memory: None,
            fast: false,
            thorough: false,
            functions: false,
            in_function: None,
            in_class: None,
            in_scope: vec![],
            imports_only: false,
            comments_only: false,
            and: vec![],
            or: vec![],
            pattern_file: None,
            lang_rules: None,
            output_filter: None,
            config: None,
            interactive_mode: false,
            fuzzy: false,
            live: false,
            check_regex: None,
            command: None,
        }
    }
}

impl Cli {
    pub fn effective_threads(&self) -> usize {
        self.threads.unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4)
        })
    }

    pub fn effective_context(&self) -> (usize, usize) {
        let before = self.before_context.unwrap_or(self.context);
        let after = self.after_context.unwrap_or(self.context);
        (before, after)
    }

    pub fn has_replacement(&self) -> bool {
        self.replace.is_some()
    }

    pub fn should_use_color(&self) -> bool {
        match self.color {
            ColorChoice::Always => true,
            ColorChoice::Never => false,
            ColorChoice::Auto => console::Term::stdout().features().colors_supported(),
        }
    }

    pub fn is_structured_search(&self) -> bool {
        self.functions
            || self.in_function.is_some()
            || self.in_class.is_some()
            || !self.in_scope.is_empty()
            || self.imports_only
            || self.comments_only
    }
}