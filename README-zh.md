<h1 align="center">ExoMind Cell</h1>

<p align="center">
  <strong>认知生命科学的操作闭合虚拟机</strong>
</p>

<p align="center">
  <a href="https://github.com/exomind-team/exomind-cell/actions"><img src="https://github.com/exomind-team/exomind-cell/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-CCOPL--1.0-blue" alt="License: CCOPL-1.0"></a>
  <a href="https://github.com/exomind-team/exomind-cell/releases"><img src="https://img.shields.io/github/v/release/exomind-team/exomind-cell" alt="Release"></a>
  <a href="https://doi.org/10.5281/zenodo.19221442"><img src="https://zenodo.org/badge/DOI/10.5281/zenodo.19221442.svg" alt="DOI"></a>
</p>

<p align="center">
  一个最小人工生命虚拟机，测试<strong>代谢新鲜度衰减</strong>（迫使有机体主动维持自身身体的约束）<br>
  是否驱动条件性生存优先行为的进化。
</p>

<p align="center">
  <a href="README.md">English</a> | 中文
</p>

<p align="center">
  <img src="docs/screenshots/tui-stigmergy.png" width="800" alt="TUI 截图">
</p>

---

## 论文

> **认知生命科学：操作闭合框架的初步实验验证**
>
> 林嘉濠. (2026). *预印本.* DOI: [10.5281/zenodo.19221442](https://doi.org/10.5281/zenodo.19221442)

## 引用本项目

如果您在研究中使用了 ExoMind Cell，请引用：

```bibtex
@article{lin2026cognitive,
  title     = {Cognitive Life Science: Preliminary Experimental Validation of the Operational Closure Framework},
  author    = {Lin, JiaHao},
  year      = {2026},
  publisher = {Zenodo},
  doi       = {10.5281/zenodo.19221442},
  url       = {https://doi.org/10.5281/zenodo.19221442}
}
```

GB/T 7714 格式：

> [1] 林嘉濠. 认知生命科学：操作闭合框架的初步实验验证[R/OL]. Zenodo, 2026. https://doi.org/10.5281/zenodo.19221442.

---

## 快速开始

```bash
# 交互式 TUI
cargo run --release -- --tui --cell          # Cell v3 模式（推荐）
cargo run --release -- --tui                 # 经典 v2 模式

# GUI spike（egui + wgpu 后端）
cargo run -- --gui                           # 原生汤视图原型
cargo run -- --gui --gui-smoke-test 5        # 打开后自动关闭的 smoke test

# 无界面实验
cargo run --release -- --cell                # v3 cell 实验（5 种子，500k ticks）
cargo run --release -- --stats               # 100 种子并行分析（2M ticks）
cargo run --release -- --run-v2              # v2 全局能量实验

# 测试
cargo test                                   # 33 个单元测试
```

## 功能特性

- **双 VM 架构**：v2（全局能量）和 v3（per-cell freshness，含 Code/Energy/Stomach/Data 四种 cell）
- **14 条指令集**：NOP, INC, DEC, CMP, JMP, JNZ, LOAD, STORE, SENSE_SELF, EAT, REFRESH, DIVIDE, EMIT, SAMPLE
- **Stigmergy 通信**：共享化学介质的间接通信机制
- **交互式 TUI**：实时可视化，支持暂停、单步、变速、个体查看
- **GUI spike**：`egui + wgpu` 汤视图，位置仅存在于 GUI 布局层
- **统计分析**：bootstrap CI、Mann-Whitney U、Kolmogorov-Smirnov 检验
- **并行执行**：rayon 多核并行实验
- **100 种子验证**：所有行为指标 p<0.001

## GUI Spike 现状

当前 GUI 原型是一个**非语义空间（non-semantic space，非真实空间语义）**的汤视图：

- `CellWorld` 不增加空间坐标，不修改 VM 生存逻辑
- 个体位置仅存在于 GUI 的力导向布局辅助层
- 渲染通过 `eframe` 走 `wgpu` 后端
- `Pause` 现在会同时冻结 VM tick 和力导向布局
- 画面内新增性能 overlay，可显示 CPU 帧时间 / 模拟 / 布局 / UI 时间
- 圆圈编码规则：
  - 颜色 = 能量
  - 半径 = cell 数量
  - 透明度 = freshness

当前渲染路径是 `egui painter on wgpu`。代码里也预留了后续升级到自定义 `wgpu render pass` 和 compute 布局的边界。

### GUI 使用说明

- `Pause`：停止模拟推进，并冻结汤视图布局。
- `Step`：在暂停状态下推进一个模拟步。
- `Ticks / frame`：数值越大，进化越快，但 CPU 压力越高，流畅度可能下降。
- 红色闪烁通常表示低能量 / 低 freshness 的个体在快速变化，不一定是渲染 bug。
- 个体跑到边缘主要是 GUI 布局平衡结果，不是 VM 里的真实空间移动。
- 若中文仍显示异常，可设置环境变量 `EXOMIND_CELL_UI_FONT` 指向一个本机 CJK 字体文件（例如 `C:\Windows\Fonts\NotoSansSC-VF.ttf`）。

## TUI 控制

| 按键 | 功能 |
|------|------|
| `p` | 暂停 / 继续 |
| `q` | 退出 |
| `h` | 帮助覆盖层 |
| `s` | 单步执行（暂停时） |
| `i` | 查看个体（cell 级别详情） |
| `<` `>` | 切换查看的个体 |
| `+` `-` | 加速 / 减速（1-1000x） |

## 核心实验结果

### 100 种子大规模实验（Cell v3，2M ticks）

| 指标 | 实验组 | 对照组 | p 值 | Cohen's d |
|------|--------|--------|------|----------|
| REFRESH 比例 | 16.8% ± 7.3% | 13.7% ± 0.4% | <0.0001 | 0.59 |
| EAT 比例 | 22.6% ± 8.7% | 15.3% ± 0.4% | <0.0001 | 1.19 |
| 种群 | 114 ± 36 | 135 ± 6 | 0.0001 | -0.78 |
| 平均能量 | 33.9 ± 17.6 | 49.2 ± 2.4 | <0.0001 | -1.22 |

REFRESH 95% CI [0.016, 0.044] 排除零——操作闭合效应在统计上得到确认。

详见 [RESULTS.md](RESULTS.md)。实验注册表：[docs/experiments.md](docs/experiments.md)。

## 项目结构

```
src/
  instruction.rs  -- 14 条指令集 + 随机/突变/显示
  organism.rs     -- v2 有机体、配置、种子程序（A/B/C）
  world.rs        -- v2 世界模拟引擎
  experiment.rs   -- v2 实验运行器、报告生成
  cell_vm.rs      -- v3 Cell 虚拟机（Cell 类型、CellOrganism、CellWorld）
  gui.rs          -- egui/wgpu 原生 GUI spike
  soup.rs         -- 仅供 GUI 使用的汤布局与渲染快照辅助层
  stats.rs        -- 统计检验（bootstrap、Mann-Whitney、KS）
  tui.rs          -- ratatui 终端可视化（v2 + Cell v3）
  main.rs         -- CLI 入口
data/             -- 实验 CSV 文件和基因组转储
docs/
  design.md       -- VM 架构设计
  experiments.md  -- 实验注册表（EXP-001 至 EXP-007）
  gui-design-proposal.md -- 未来 GUI 设计方案（egui/wgpu）
```

## 文档

- [VM 设计](docs/design.md) — 指令集、有机体结构、cell 类型
- [实验注册表](docs/experiments.md) — 全部 7 个实验的参数和结果
- [GUI 方案](docs/gui-design-proposal.md) — 未来图形界面设计
- [GUI Spike 计划](docs/plans/2026-03-26-egui-wgpu-soup-view.md) — 当前原型的实现计划

## 技术栈

- **语言**：Rust 2021
- **依赖**：`rand 0.8`、`rayon 1.10`、`ratatui 0.29`、`crossterm 0.28`、`eframe 0.31`
- **测试**：33 个单元测试
- **CI**：GitHub Actions（构建 + 测试 + clippy）

## 许可证

[CCOPL-1.0](LICENSE)（贡献者集体所有制公共许可证）
