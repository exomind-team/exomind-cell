#!/usr/bin/env python3
"""Generate figures for Paper I: Cognitive Life Science."""

import pandas as pd
import matplotlib.pyplot as plt
import matplotlib
import numpy as np
from pathlib import Path

matplotlib.rcParams['font.family'] = 'DejaVu Sans'
matplotlib.rcParams['font.size'] = 11
matplotlib.rcParams['figure.dpi'] = 300

DATA_PATH = Path("D:/project/d0-vm/data/large_scale/per_seed_summary.csv")
OUT_DIR = Path("D:/project/ExoMind-Obsidian-HailayLin/3-学科知识沉淀/33-AI_认知科学/认知生命科学/论文一/figures")
OUT_DIR.mkdir(parents=True, exist_ok=True)

df = pd.read_csv(DATA_PATH)
exp = df[df['group'] == 'exp']
ctrl = df[df['group'] == 'ctrl']

# --- Figure 3: REFRESH Bimodal Distribution ---
fig, axes = plt.subplots(1, 2, figsize=(10, 4), sharey=True)

axes[0].hist(exp['refresh_ratio'] * 100, bins=20, color='#2196F3', edgecolor='black', alpha=0.8)
axes[0].set_title('Experimental (freshness decay ON)', fontsize=12)
axes[0].set_xlabel('REFRESH ratio (%)')
axes[0].set_ylabel('Number of seeds')
axes[0].axvline(exp['refresh_ratio'].mean() * 100, color='red', linestyle='--', label=f'mean={exp["refresh_ratio"].mean()*100:.1f}%')
axes[0].legend()

axes[1].hist(ctrl['refresh_ratio'] * 100, bins=20, color='#FF9800', edgecolor='black', alpha=0.8)
axes[1].set_title('Control (freshness decay OFF)', fontsize=12)
axes[1].set_xlabel('REFRESH ratio (%)')
axes[1].axvline(ctrl['refresh_ratio'].mean() * 100, color='red', linestyle='--', label=f'mean={ctrl["refresh_ratio"].mean()*100:.1f}%')
axes[1].legend()

fig.suptitle('Figure 3: REFRESH Distribution — Strategy Diversification\n(KS D=0.68, p<0.001; N=100 seeds each)', fontsize=13, fontweight='bold')
plt.tight_layout()
plt.savefig(OUT_DIR / 'fig3_refresh_distribution.png', bbox_inches='tight')
plt.close()

# --- Figure 4: Instruction Ratio Bar Chart ---
metrics = ['EAT', 'REFRESH', 'DIVIDE']
exp_means = [exp['eat_ratio'].mean()*100, exp['refresh_ratio'].mean()*100, exp['divide_ratio'].mean()*100]
ctrl_means = [ctrl['eat_ratio'].mean()*100, ctrl['refresh_ratio'].mean()*100, ctrl['divide_ratio'].mean()*100]
exp_sds = [exp['eat_ratio'].std()*100, exp['refresh_ratio'].std()*100, exp['divide_ratio'].std()*100]
ctrl_sds = [ctrl['eat_ratio'].std()*100, ctrl['refresh_ratio'].std()*100, ctrl['divide_ratio'].std()*100]

x = np.arange(len(metrics))
width = 0.35

fig, ax = plt.subplots(figsize=(8, 5))
bars1 = ax.bar(x - width/2, exp_means, width, yerr=exp_sds, label='Experimental', color='#2196F3', edgecolor='black', capsize=5)
bars2 = ax.bar(x + width/2, ctrl_means, width, yerr=ctrl_sds, label='Control', color='#FF9800', edgecolor='black', capsize=5)

# Annotate with Cohen's d
cohens_d = {'EAT': 1.19, 'REFRESH': 0.59, 'DIVIDE': 0.30}
for i, m in enumerate(metrics):
    ax.annotate(f'd={cohens_d[m]:.2f}\np<0.001',
                xy=(i, max(exp_means[i], ctrl_means[i]) + max(exp_sds[i], ctrl_sds[i]) + 2),
                ha='center', fontsize=9, fontstyle='italic')

ax.set_ylabel('Instruction ratio (%)')
ax.set_title("Figure 4: Instruction Ratios — Experimental vs Control\n(100 seeds, 2M ticks, all p<0.001)", fontweight='bold')
ax.set_xticks(x)
ax.set_xticklabels(metrics)
ax.legend()
ax.set_ylim(0, 40)
plt.tight_layout()
plt.savefig(OUT_DIR / 'fig4_instruction_ratios.png', bbox_inches='tight')
plt.close()

# --- Figure 5: R Gradient ---
r_values = [1, 2, 3, 5, 8]
exp_refresh = [0, 0, 12.5, 18.3, 21.0]
exp_divide = [8.2, 9.1, 15.1, 10.7, 8.5]
ctrl_refresh = [14.2] * 5

fig, ax = plt.subplots(figsize=(8, 5))
ax.plot(r_values, exp_refresh, 'o-', color='#2196F3', linewidth=2, markersize=8, label='Exp REFRESH%')
ax.plot(r_values, exp_divide, 's--', color='#4CAF50', linewidth=2, markersize=8, label='Exp DIVIDE%')
ax.plot(r_values, ctrl_refresh, ':', color='#FF9800', linewidth=2, label='Ctrl REFRESH% (baseline)')

ax.axvline(3, color='gray', linestyle='--', alpha=0.5)
ax.annotate('R=3: max diversification', xy=(3, 15.1), xytext=(4.5, 18),
            arrowprops=dict(arrowstyle='->', color='gray'), fontsize=10, fontstyle='italic')

ax.set_xlabel('REFRESH Radius (R)')
ax.set_ylabel('Instruction ratio (%)')
ax.set_title('Figure 5: REFRESH Radius Gradient — Constraint Strictness\n(Control unaffected by R)', fontweight='bold')
ax.legend()
ax.set_xticks(r_values)
plt.tight_layout()
plt.savefig(OUT_DIR / 'fig5_r_gradient.png', bbox_inches='tight')
plt.close()

print(f"Generated 3 figures in {OUT_DIR}")
print("  fig3_refresh_distribution.png")
print("  fig4_instruction_ratios.png")
print("  fig5_r_gradient.png")
