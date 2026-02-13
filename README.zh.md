# temci

高级基准测试工具 - Rust 实现

**语言 / Language:**
[English](README.md) | [简体中文](README.zh.md) | [繁體粵語](README.zh-yue.md) | [日本語](README.ja.md)

## 简介

**temci**（Timing Execution Measuring Control and Inspection，计时执行测量控制与检查）是一个全面的基准测试工具，用于测量和分析命令执行性能。这是原 Python 工具的 Rust 实现，提供了更高的性能和可靠性。

temci 允许您：
- 精确测量命令执行时间
- 使用可配置参数运行基准测试（运行次数、超时、CPU 亲和性）
- 生成多种格式的详细报告（文本、Markdown、JSON、CSV）
- 对基准测试结果进行统计分析
- 控制 CPU 亲和性和进程隔离
- 构建和编译基准测试程序

## 功能特性

### 核心基准测试
- **精确计时**：使用墙钟时间和 CPU 时间进行高精度计时
- **多种运行器**：支持基本、性能计数器和自定义运行驱动器
- **并行执行**：支持并发基准测试执行的工作线程池
- **可配置参数**：运行次数、超时、预热运行、显示输出选项

### 统计分析
- 平均值、中位数、标准差计算
- 百分位数计算（p50、p90、p95、p99）
- 异常值检测和过滤
- 置信区间计算

### 报告生成
- 多种输出格式：控制台/文本、Markdown、JSON、CSV
- 可自定义的报告布局
- 保存和加载基准测试结果
- 运行之间的比较报告

### 进程控制
- CPU 亲和性控制（绑定到特定核心）
- 进程隔离和优先级控制
- 环境变量管理
- 工作目录指定

### 构建系统
- 支持 GCC、Clang 和 Rust 编译器检测
- 自动编译器检测和优化级别选择
- 与基准测试执行集成

### Shell 集成
- Bash 自动补全支持
- Shell 输出格式化

## 安装

### 从源代码安装

```bash
git clone https://github.com/iMMIQ/temci_rs.git
cd temci_rs
cargo build --release
sudo install target/release/temci /usr/local/bin/
```

### 使用 Cargo 安装

```bash
cargo install temci
```

## 使用方法

### 基本执行

执行命令并测量其执行时间：

```bash
temci short-exec 'echo "Hello, World!"'
```

运行多次执行的基准测试：

```bash
temci short-exec --runs 20 'sleep 1'
```

**注意**：`short-exec` 会自动将结果保存到 `temci_results.json`，以便后续使用 `report` 命令。使用 `--no-save` 选项可以禁用此行为。

### 基准测试配置

创建 YAML 配置文件（`temci.yaml`）：

```yaml
benchmarks:
  - name: "睡眠测试"
    command: "sleep"
    args: ["0.1"]
    runs: 10
    timeout: 5

  - name: "命令测试"
    command: "echo"
    args: ["test"]
    runs: 5
    show_output: true
```

从配置文件运行基准测试：

```bash
temci exec
```

### 报告生成

从保存的结果生成报告：

```bash
# 控制台输出（默认）- 从 temci_results.json 读取
temci report

# 保存到文件
temci report --output results.txt

# 不同格式
temci report --format json --output results.json
temci report --format csv --output results.csv
temci report --format markdown --output results.md

# 使用自定义输入文件
temci report --input my_results.yaml
```

### 构建集成

编译和基准测试程序：

```bash
# 设置构建环境
temci setup

# 使用检测到的编译器构建
temci build --compiler gcc --opt-level O3
```

### Shell 自动补全

启用 bash 自动补全：

```bash
# 生成补全文件
temci completion bash > /etc/bash_completion.d/temci

# 或直接 source
source <(temci completion bash)
```

### 清理

删除基准测试工件和缓存：

```bash
# 清理默认工件
temci clean

# 清理所有工件包括缓存
temci clean --all
```

## 命令参考

### `short-exec`
快速执行并计时测量。
```bash
temci short-exec [OPTIONS] <COMMANDS>...
```

选项：
- `-r, --runs <RUNS>` - 执行次数（默认：10）
- `-w, --warmup <WARMUP>` - 预热运行次数（默认：0）
- `-S, --summary` - 仅显示摘要
- `-o, --output <OUTPUT>` - 结果输出文件（默认：temci_results.json）
- `--no-save` - 不保存结果到文件

### `exec`
完整的基准测试执行，支持配置文件。
```bash
temci exec [OPTIONS]
```

### `report`
从保存的基准测试结果生成报告。
```bash
temci report [OPTIONS] [--input INPUT]
```

选项：
- `-f, --format <FORMAT>` - 报告类型：console、csv、json、markdown（默认：console）
- `-o, --output <OUTPUT>` - 输出文件（默认：标准输出）
- `-i, --input <INPUT>` - 输入数据文件（默认：temci_results.json）

### `build`
编译程序用于基准测试。
```bash
temci build [OPTIONS]
```

### `clean`
删除基准测试工件。
```bash
temci clean [OPTIONS]
```

### `setup`
初始配置和工具检测。
```bash
temci setup [OPTIONS]
```

### `completion`
生成 Shell 自动补全脚本。
```bash
temci completion <SHELL>          # 位置参数
temci completion -s <SHELL>        # 使用 -s 选项
temci completion --shell <SHELL>  # 使用 --shell 选项
```

支持的 Shell：`bash`、`zsh`、`fish`、`elvish`、`powershell`

## 配置文件格式

temci 使用 YAML 配置文件处理复杂的基准测试场景：

```yaml
# 全局设置
runs: 100
timeout: 60
show_output: false
show_stderr: false
unscaled_stdout: true
work_dir: /path/to/benchmarks

# 环境变量
env:
  RUST_BACKTRACE: "1"
  CUSTOM_VAR: "value"

# CPU 亲和性（逗号分隔列表或范围）
cpuset: "0-3"  # 使用核心 0、1、2、3

# 单个基准测试
benchmarks:
  - name: "基准测试 1"
    command: "program"
    args: ["--arg1", "value"]
    runs: 50
    warmup: 5

  - name: "基准测试 2"
    command: "program"
    args: ["--arg2", "value"]
    env:
      SPECIAL_VAR: "special_value"
```

## 结果文件

基准测试结果以 JSON 或 YAML 格式保存：

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

## 许可证

GPL-3.0

## 贡献

欢迎贡献！请提交拉取请求或提出问题以报告 bug 和功能请求。

## 从 Python 版本迁移

此 Rust 实现提供：
- **更好的性能**：更快的执行速度和更低的开销
- **类型安全**：编译时保证防止运行时错误
- **现代 CLI**：增强的命令行界面和更好的帮助文本
- **追踪**：使用 `tracing` crate 的结构化日志

有关详细功能比较，请参阅 COMPARISON.md。
