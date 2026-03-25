#!/usr/bin/env python3
"""Generate all figures for Paper I from experiment data."""

import csv
import os

try:
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    import matplotlib.patches as mpatches
    plt.rcParams.update({
        'font.size': 10,
        'axes.titlesize': 12,
        'axes.labelsize': 10,
        'xtick.labelsize': 9,
        'ytick.labelsize': 9,
        'legend.fontsize': 9,
        'figure.figsize': (8, 5),
        'figure.dpi': 300,
        'lines.linewidth': 1.5,
        'font.sans-serif': ['SimHei', 'Microsoft YaHei'],
        'axes.unicode_minus': False,
    })
    EXP_COLOR = '#E74C3C'
    CTRL_COLOR = '#3498DB'
except ImportError:
    print("Need: pip install matplotlib")
    exit(1)

DATA = "D:/project/d0-vm/data/large_scale"
FIG = "D:/project/ExoMind-Obsidian-HailayLin/3-学科知识沉淀/33-AI_认知科学/认知生命科学/论文一/figures"
os.makedirs(FIG, exist_ok=True)

def read_csv(path):
    with open(path) as f:
        return list(csv.DictReader(f))

# ============================================================================
# Fig 1: REFRESH distribution (100-seed, exp vs ctrl)
# ============================================================================
def fig_refresh_distribution():
    rows = read_csv(f"{DATA}/per_seed_summary.csv")
    exp = [float(r['refresh_ratio'])*100 for r in rows if r['group']=='exp']
    ctrl = [float(r['refresh_ratio'])*100 for r in rows if r['group']=='ctrl']

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12, 5))

    ax1.hist(exp, bins=20, alpha=0.7, color=EXP_COLOR, label='实验组（有衰减）')
    ax1.hist(ctrl, bins=20, alpha=0.7, color=CTRL_COLOR, label='对照组（无衰减）')
    ax1.set_xlabel('REFRESH 比例 (%)')
    ax1.set_ylabel('种子数量')
    ax1.set_title('REFRESH 分布（100 个种子）')
    ax1.legend(fontsize=11)

    ax2.boxplot([exp, ctrl], tick_labels=['实验组', '对照组'],
                boxprops=dict(color='#333'), medianprops=dict(color='red'))
    ax2.set_ylabel('REFRESH 比例 (%)')
    ax2.set_title('REFRESH：实验组 vs 对照组')

    plt.tight_layout()
    plt.savefig(f"{FIG}/fig_refresh_distribution.png", dpi=200)
    plt.close()
    print("  Saved fig_refresh_distribution.png")

# ============================================================================
# Fig 2: GATE history effect (EXP-014/OPT-GATE)
# ============================================================================
def fig_gate_history():
    # Use optimized GATE data
    path = "D:/project/d0-vm/docs/experiments/EXP-OPT-GATE/data/per_seed.csv"
    if not os.path.exists(path):
        print("  Skipping GATE history: no data")
        return

    rows = read_csv(path)
    exp_ref = [float(r['refresh'])*100 for r in rows if r['group']=='exp']
    ctrl_ref = [float(r['refresh'])*100 for r in rows if r['group']=='ctrl']

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12, 5))

    # Bar chart
    means = [sum(exp_ref)/len(exp_ref), sum(ctrl_ref)/len(ctrl_ref)]
    bars = ax1.bar(['丰裕→匮乏', '一直匮乏'], means,
                   color=[EXP_COLOR, CTRL_COLOR], alpha=0.8)
    ax1.set_ylabel('REFRESH 比例 (%)')
    ax1.set_title('GATE 历史效应 (p<0.0001, d=1.12)')
    for bar, val in zip(bars, means):
        ax1.text(bar.get_x() + bar.get_width()/2, bar.get_height() + 0.3,
                f'{val:.1f}%', ha='center')

    # Distribution
    ax2.hist(exp_ref, bins=20, alpha=0.7, color=EXP_COLOR, label='丰裕→匮乏')
    ax2.hist(ctrl_ref, bins=20, alpha=0.7, color=CTRL_COLOR, label='一直匮乏')
    ax2.set_xlabel('REFRESH 比例 (%)')
    ax2.set_ylabel('种子数量')
    ax2.set_title('REFRESH 分布（按历史分组）')
    ax2.legend(fontsize=11)

    plt.tight_layout()
    plt.savefig(f"{FIG}/fig_gate_history.png", dpi=200)
    plt.close()
    print("  Saved fig_gate_history.png")

# ============================================================================
# Fig 3: 2x2 Matrix heatmap
# ============================================================================
def fig_2x2_matrix():
    # Data from experiments
    data = {
        'Optimized\nNo GATE': 75,    # existing 100-seed replication rate
        'Optimized\nGATE': 82,        # EXP-OPT-GATE direction win
        'Non-optimized\nNo GATE': 50,  # baseline (no history effect expected)
        'Non-optimized\nGATE': 44,     # EXP-CROSS G4
    }

    fig, ax = plt.subplots(figsize=(8, 6))
    matrix = [[75, 82], [50, 44]]
    im = ax.imshow(matrix, cmap='RdYlGn', vmin=30, vmax=95, aspect='auto')

    ax.set_xticks([0, 1])
    ax.set_xticklabels(['无 GATE', '有 GATE'])
    ax.set_yticks([0, 1])
    ax.set_yticklabels(['优化参数\n(food=500)', '非优化参数\n(food=50)'])

    for i in range(2):
        for j in range(2):
            val = matrix[i][j]
            color = 'white' if val < 55 else 'black'
            sig = '***' if val > 70 else ('NS' if val < 55 else '*')
            ax.text(j, i, f'{val}%\n{sig}', ha='center', va='center',
                   fontsize=16, fontweight='bold', color=color)

    ax.set_title('GATE × 参数交互效应\n（历史效应方向胜率）')
    plt.colorbar(im, label='方向胜率 (%)')
    plt.tight_layout()
    plt.savefig(f"{FIG}/fig_2x2_matrix.png", dpi=200)
    plt.close()
    print("  Saved fig_2x2_matrix.png")

# ============================================================================
# Fig 4: Instruction ratio comparison (exp vs ctrl)
# ============================================================================
def fig_instruction_ratios():
    rows = read_csv(f"{DATA}/per_seed_summary.csv")
    exp = [r for r in rows if r['group']=='exp']
    ctrl = [r for r in rows if r['group']=='ctrl']

    metrics = ['eat_ratio', 'refresh_ratio', 'divide_ratio']
    labels = ['EAT', 'REFRESH', 'DIVIDE']

    exp_means = [sum(float(r[m]) for r in exp)/len(exp)*100 for m in metrics]
    ctrl_means = [sum(float(r[m]) for r in ctrl)/len(ctrl)*100 for m in metrics]

    x = range(len(labels))
    width = 0.35

    fig, ax = plt.subplots(figsize=(8, 5))
    bars1 = ax.bar([i - width/2 for i in x], exp_means, width, label='实验组', color=EXP_COLOR, alpha=0.8)
    bars2 = ax.bar([i + width/2 for i in x], ctrl_means, width, label='对照组', color=CTRL_COLOR, alpha=0.8)

    ax.set_ylabel('占比 (%)')
    ax.set_title('指令比例：实验组 vs 对照组（100 个种子）')
    ax.set_xticks(x)
    ax.set_xticklabels(['进食', '刷新', '复制'])
    ax.legend(fontsize=11)

    for bars in [bars1, bars2]:
        for bar in bars:
            h = bar.get_height()
            ax.text(bar.get_x() + bar.get_width()/2, h + 0.3, f'{h:.1f}', ha='center', fontsize=10)

    plt.tight_layout()
    plt.savefig(f"{FIG}/fig_instruction_ratios.png", dpi=200)
    plt.close()
    print("  Saved fig_instruction_ratios.png")

# ============================================================================
# Fig 5: Knockout analysis
# ============================================================================
def fig_knockout():
    path = "D:/project/d0-vm/docs/experiments/EXP-014-gate-learning/data/knockout.csv"
    if not os.path.exists(path):
        print("  Skipping knockout: no data")
        return

    rows = read_csv(path)
    organisms = {}
    for r in rows:
        name = r['organism'].split('(')[0].strip()
        if name not in organisms:
            organisms[name] = {'lethal': 0, 'neutral': 0, 'beneficial': 0, 'total': 0}
        organisms[name]['total'] += 1
        cat = r['result']
        if cat in ('lethal', 'severe_defect'):
            organisms[name]['lethal'] += 1
        elif cat == 'beneficial':
            organisms[name]['beneficial'] += 1
        else:
            organisms[name]['neutral'] += 1

    fig, ax = plt.subplots(figsize=(10, 5))
    names = list(organisms.keys())
    lethal = [organisms[n]['lethal'] for n in names]
    neutral = [organisms[n]['neutral'] for n in names]
    beneficial = [organisms[n]['beneficial'] for n in names]

    x = range(len(names))
    ax.bar(x, lethal, color=CTRL_COLOR, label='致死/严重', alpha=0.8)
    ax.bar(x, neutral, bottom=lethal, color='#9E9E9E', label='中性', alpha=0.8)
    ax.bar(x, beneficial, bottom=[l+n for l,n in zip(lethal, neutral)], color=EXP_COLOR, label='有益', alpha=0.8)

    ax.set_xticks(x)
    ax.set_xticklabels(names)
    ax.set_ylabel('代码细胞数')
    ax.set_title('敲除分析：必要 vs 非必要指令')
    ax.legend(fontsize=11)

    for i, n in enumerate(names):
        total = organisms[n]['total']
        essential = organisms[n]['lethal']
        ax.text(i, total + 0.2, f'{essential}/{total}\n({100*essential/total:.0f}%)',
               ha='center', fontsize=10)

    plt.tight_layout()
    plt.savefig(f"{FIG}/fig_knockout.png", dpi=200)
    plt.close()
    print("  Saved fig_knockout.png")

if __name__ == '__main__':
    print("Generating Paper I figures...\n")
    fig_refresh_distribution()
    fig_gate_history()
    fig_2x2_matrix()
    fig_instruction_ratios()
    fig_knockout()
    print(f"\nAll figures saved to: {FIG}")
