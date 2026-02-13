# temci

高度なベンチマークツール - Rust実装

**言語 / Language:**
[English](README.md) | [简体中文](README.zh.md) | [繁體粵語](README.zh-yue.md) | [日本語](README.ja.md)

## 説明

**temci**（Timing Execution Measuring Control and Inspection）は、コマンドの実行パフォーマンスを測定・分析するための包括的なベンチマークツールです。これは元のPythonツールのRust実装であり、高いパフォーマンスと信頼性を提供します。

temciを使用すると、以下のことができます：
- 正確なタイミング測定でコマンドを実行
- 設定可能なパラメータ（実行回数、タイムアウト、CPUアフィニティ）でベンチマークを実行
- 複数の形式で詳細なレポートを生成（テキスト、Markdown、JSON、CSV）
- ベンチマーク結果の統計分析
- CPUアフィニティとプロセス分離の制御
- ベンチマークプログラムのビルドとコンパイル

## 機能

### コアベンチマーク
- **正確なタイミング**：ウォールクロック時間とCPU時間を使用した高精度タイミング測定
- **複数のランナー**：ベーシック、パフォーマンスカウンター、カスタムランナーをサポート
- **並列実行**：同時ベンチマーク実行のためのワーカープール
- **設定可能なパラメータ**：実行回数、タイムアウト、ウォームアップ実行、出力表示オプション

### 統計分析
- 平均値、中央値、標準偏差の計算
- パーセンタイル計算（p50、p90、p95、p99）
- 外れ値の検出とフィルタリング
- 信頼区間の計算

### レポート作成
- 複数の出力形式：コンソール/テキスト、Markdown、JSON、CSV
- カスタマイズ可能なレイアウト
- ベンチマーク結果の保存と読み込み
- 実行間の比較レポート

### プロセス制御
- CPUアフィニティ制御（特定のコアに固定）
- プロセス分離と優先度制御
- 環境変数管理
- 作業ディレクトリの指定

### ビルドシステム
- GCC、Clang、Rustコンパイラの検出をサポート
- 自動コンパイラ検出と最適化レベル選択
- ベンチマーク実行との統合

### シェル統合
- Bash補完サポート
- シェル出力のフォーマット

## インストール

### ソースから

```bash
git clone https://github.com/iMMIQ/temci_rs.git
cd temci_rs
cargo build --release
sudo install target/release/temci /usr/local/bin/
```

### Cargoを使用

```bash
cargo install temci
```

## 使用方法

### 基本的な実行

コマンドを実行して実行時間を測定します：

```bash
temci short-exec 'echo "Hello, World!"'
```

複数回実行するベンチマークを実行します：

```bash
temci short-exec --runs 20 'sleep 1'
```

**注意**: `short-exec` は自動的に結果を `temci_results.json` に保存し、後で `report` コマンドで使用できるようにします。この動作を無効にするには `--no-save` オプションを使用してください。

### ベンチマーク設定

YAML設定ファイル（`temci.yaml`）を作成します：

```yaml
benchmarks:
  - name: "スリープテスト"
    command: "sleep"
    args: ["0.1"]
    runs: 10
    timeout: 5

  - name: "コマンドテスト"
    command: "echo"
    args: ["test"]
    runs: 5
    show_output: true
```

設定からベンチマークを実行します：

```bash
temci exec
```

### レポート生成

保存された結果からレポートを生成します：

```bash
# コンソール出力（デフォルト）- temci_results.json から読み込み
temci report

# ファイルに保存
temci report --output results.txt

# 異なる形式
temci report --format json --output results.json
temci report --format csv --output results.csv
temci report --format markdown --output results.md

# カスタム入力ファイルを使用
temci report --input my_results.yaml
```

### ビルド統合

プログラムをコンパイルしてベンチマークします：

```bash
# ビルド環境をセットアップ
temci setup

# 検出されたコンパイラでビルド
temci build --compiler gcc --opt-level O3
```

### シェル補完

bash補完を有効にします：

```bash
# 補完ファイルを生成
temci completion bash > /etc/bash_completion.d/temci

# または直接ソース
source <(temci completion bash)
```

### クリーンアップ

ベンチマークのアーティファクトとキャッシュを削除します：

```bash
# デフォルトのアーティファクトをクリーンアップ
temci clean

# キャッシュを含むすべてのアーティファクトをクリーンアップ
temci clean --all
```

## コマンドリファレンス

### `short-exec`
タイミング測定付きの高速実行。
```bash
temci short-exec [OPTIONS] <COMMANDS>...
```

オプション：
- `-r, --runs <RUNS>` - 実行回数（デフォルト：10）
- `-w, --warmup <WARMUP>` - ウォームアップ実行回数（デフォルト：0）
- `-S, --summary` - サマリーのみ表示
- `-o, --output <OUTPUT>` - 結果の出力ファイル（デフォルト：temci_results.json）
- `--no-save` - 結果をファイルに保存しない

### `exec`
設定ファイルサポートを備えた完全なベンチマーク実行。
```bash
temci exec [OPTIONS]
```

### `report`
保存されたベンチマーク結果からレポートを生成。
```bash
temci report [OPTIONS] [--input INPUT]
```

オプション：
- `-f, --format <FORMAT>` - レポートタイプ：console、csv、json、markdown（デフォルト：console）
- `-o, --output <OUTPUT>` - 出力ファイル（デフォルト：標準出力）
- `-i, --input <INPUT>` - 入力データファイル（デフォルト：temci_results.json）

### `build`
ベンチマーク用のプログラムをコンパイル。
```bash
temci build [OPTIONS]
```

### `clean`
ベンチマークのアーティファクトを削除。
```bash
temci clean [OPTIONS]
```

### `setup`
初期設定とツール検出。
```bash
temci setup [OPTIONS]
```

### `completion`
シェル補完スクリプトを生成。
```bash
temci completion <SHELL>          # 位置引数
temci completion -s <SHELL>        # -s オプション使用
temci completion --shell <SHELL>  # --shell オプション使用
```

対応シェル：`bash`、`zsh`、`fish`、`elvish`、`powershell`

## 設定ファイル形式

temciは複雑なベンチマークシナリオのためにYAML設定ファイルを使用します：

```yaml
# グローバル設定
runs: 100
timeout: 60
show_output: false
show_stderr: false
unscaled_stdout: true
work_dir: /path/to/benchmarks

# 環境変数
env:
  RUST_BACKTRACE: "1"
  CUSTOM_VAR: "value"

# CPUアフィニティ（カンマ区切りリストまたは範囲）
cpuset: "0-3"  # コア 0、1、2、3 を使用

# 個別のベンチマーク
benchmarks:
  - name: "ベンチマーク1"
    command: "program"
    args: ["--arg1", "value"]
    runs: 50
    warmup: 5

  - name: "ベンチマーク2"
    command: "program"
    args: ["--arg2", "value"]
    env:
      SPECIAL_VAR: "special_value"
```

## 結果ファイル

ベンチマーク結果はJSONまたはYAML形式で保存されます：

```json
{
  "name": "benchmark suite",
  "timestamp": "2024-01-01T00:00:00Z",
  "results": [
    {
      "command": "sleep",
      "args": ["0.1"],
      "runs": 10,
      "successful": 10,
      "failed": 0,
      "min_ms": 100.5,
      "max_ms": 105.2,
      "avg_ms": 102.3,
      "total_ms": 1023.0
    }
  ]
}
```

## ライセンス

GPL-3.0

## 貢献

貢献を歓迎します！プルリクエストを送信するか、バグや機能リクエストについてはIssueを開いてください。

## Python版からの移行

このRust実装は以下を提供します：
- **より良いパフォーマンス**：より高速な実行と低いオーバーヘッド
- **型安全性**：実行時エラーを防ぐコンパイル時保証
- **モダンなCLI**：改善されたコマンドラインインターフェースとヘルプテキスト
- **トレース**：`tracing`クレートを使用した構造化ログ

詳細な機能比較については、COMPARISON.mdを参照してください。
