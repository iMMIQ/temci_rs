# temci

高級基準測試工具 - Rust 實現

**語言 / Language:**
[English](README.md) | [简体中文](README.zh.md) | [繁體粵語](README.zh-yue.md) | [日本語](README.ja.md)

## 簡介

**temci**（Timing Execution Measuring Control and Inspection，計時執行測量控制與檢查）係一個全面嘅基準測試工具，用嚟測量同分析指令執行性能。呢個係原 Python 工具嘅 Rust 實現，提供更高嘅性能同可靠性。

temci 可以令你：
- 精確測量指令執行時間
- 用可設定參數運行基準測試（運行次數、逾時、CPU 親和性）
- 生成多種格式嘅詳細報告（文字、Markdown、JSON、CSV）
- 對基準測試結果做統計分析
- 控制 CPU 親和性同進程隔離
- 構建同編譯基準測試程式

## 功能特性

### 核心基準測試
- **精確計時**：使用牆鐘時間同 CPU 時間做高精度計時
- **多種運行器**：支援基本、性能計數器同自定義運行驅動器
- **並行執行**：支援並發基準測試執行嘅工作線程池
- **可設定參數**：運行次數、逾時、預熱運行、顯示輸出選項

### 統計分析
- 平均值、中位數、標準差計算
- 百分位數計算（p50、p90、p95、p99）
- 异常值檢測同過濾
- 信賴區間計算

### 報告生成
- 多種輸出格式：控制台/文字、Markdown、JSON、CSV
- 可自訂嘅報告布局
- 保存同載入基準測試結果
- 運行之間嘅比較報告

### 進程控制
- CPU 親和性控制（綁定到特定核心）
- 進程隔離同優先級控制
- 環境變量管理
- 工作目錄指定

### 構建系統
- 支援 GCC、Clang 同 Rust 編譯器檢測
- 自動編譯器檢測同優化級別選擇
- 同基準測試執行整合

### Shell 整合
- Bash 自動補完支援
- Shell 輸出格式化

## 安裝

### 從源碼安裝

```bash
git clone https://github.com/iMMIQ/temci_rs.git
cd temci_rs
cargo build --release
sudo install target/release/temci /usr/local/bin/
```

### 使用 Cargo 安裝

```bash
cargo install temci
```

## 使用方法

### 基本執行

執行指令並測量其執行時間：

```bash
temci short-exec 'echo "Hello, World!"'
```

運行多次執行嘅基準測試：

```bash
temci short-exec --runs 20 'sleep 1'
```

**注意**：`short-exec` 會自動將結果儲存到 `temci_results.json`，以便後續使用 `report` 指令。使用 `--no-save` 選項可以禁用此行為。

### 基準測試設定

建立 YAML 設定檔（`temci.yaml`）：

```yaml
benchmarks:
  - name: "瞓覺測試"
    command: "sleep"
    args: ["0.1"]
    runs: 10
    timeout: 5

  - name: "指令測試"
    command: "echo"
    args: ["test"]
    runs: 5
    show_output: true
```

從設定檔運行基準測試：

```bash
temci exec
```

### 報告生成

從保存嘅結果生成報告：

```bash
# 控制台輸出（預設）- 從 temci_results.json 讀取
temci report

# 保存到檔案
temci report --output results.txt

# 不同格式
temci report --format json --output results.json
temci report --format csv --output results.csv
temci report --format markdown --output results.md

# 使用自訂輸入檔案
temci report --input my_results.yaml
```

### 構建整合

編譯同基準測試程式：

```bash
# 設定構建環境
temci setup

# 用檢測到嘅編譯器構建
temci build --compiler gcc --opt-level O3
```

### Shell 自動補完

啟用 bash 自動補完：

```bash
# 生成補完檔案
temci completion bash > /etc/bash_completion.d/temci

# 或直接 source
source <(temci completion bash)
```

### 清理

刪除基準測試工件同快取：

```bash
# 清理預設工件
temci clean

# 清理所有工件包括快取
temci clean --all
```

## 指令參考

### `short-exec`
快速執行並計時測量。
```bash
temci short-exec [OPTIONS] <COMMANDS>...
```

選項：
- `-r, --runs <RUNS>` - 執行次數（預設：10）
- `-w, --warmup <WARMUP>` - 預熱執行次數（預設：0）
- `-S, --summary` - 僅顯示摘要
- `-o, --output <OUTPUT>` - 結果輸出檔案（預設：temci_results.json）
- `--no-save` - 唔儲存結果到檔案

### `exec`
完整嘅基準測試執行，支援設定檔。
```bash
temci exec [OPTIONS]
```

### `report`
從保存嘅基準測試結果生成報告。
```bash
temci report [OPTIONS] [--input INPUT]
```

選項：
- `-f, --format <FORMAT>` - 報告類型：console、csv、json、markdown（預設：console）
- `-o, --output <OUTPUT>` - 輸出檔案（預設：標準輸出）
- `-i, --input <INPUT>` - 輸入數據檔案（預設：temci_results.json）

### `build`
編譯程式用於基準測試。
```bash
temci build [OPTIONS]
```

### `clean`
刪除基準測試工件。
```bash
temci clean [OPTIONS]
```

### `setup`
初始設定同工具檢測。
```bash
temci setup [OPTIONS]
```

### `completion`
生成 Shell 自動補完腳本。
```bash
temci completion <SHELL>          # 位置參數
temci completion -s <SHELL>        # 使用 -s 選項
temci completion --shell <SHELL>  # 使用 --shell 選項
```

支援嘅 Shell：`bash`、`zsh`、`fish`、`elvish`、`powershell`

## 設定檔格式

temci 用 YAML 設定檔處理複雜嘅基準測試場景：

```yaml
# 全域設定
runs: 100
timeout: 60
show_output: false
show_stderr: false
unscaled_stdout: true
work_dir: /path/to/benchmarks

# 環境變量
env:
  RUST_BACKTRACE: "1"
  CUSTOM_VAR: "value"

# CPU 親和性（逗號分隔列表或者範圍）
cpuset: "0-3"  # 使用核心 0、1、2、3

# 單個基準測試
benchmarks:
  - name: "基準測試 1"
    command: "program"
    args: ["--arg1", "value"]
    runs: 50
    warmup: 5

  - name: "基準測試 2"
    command: "program"
    args: ["--arg2", "value"]
    env:
      SPECIAL_VAR: "special_value"
```

## 結果檔案

基準測試結果以 JSON 或者 YAML 格式保存：

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

## 授權

GPL-3.0

## 貢獻

歡迎貢獻！請提交 Pull Request 或者開 Issue 嚟報告 bug 同功能請求。

## 從 Python 版本遷移

呢個 Rust 實現提供：
- **更好嘅性能**：更快嘅執行速度同更低嘅開銷
- **類型安全**：編譯時保證防止運行時錯誤
- **現代 CLI**：增強嘅指令列界面同更好嘅幫助文字
- **追蹤**：使用 `tracing` crate 嘅結構化日誌

詳細功能比較請參閱 COMPARISON.md。
