#!/usr/bin/env python3
"""Generate figures for ExoMind Cell paper from experiment CSV data."""

import csv
import os

# Try matplotlib; if not available, generate data summaries instead
try:
    import matplotlib
    matplotlib.use('Agg')  # Non-interactive backend
    import matplotlib.pyplot as plt
    HAS_MPL = True
except ImportError:
    HAS_MPL = False
    print("matplotlib not available. Install with: pip install matplotlib")
    print("Generating text summaries instead.\n")

DATA_DIR = os.path.join(os.path.dirname(__file__), '..', 'data')
FIG_DIR = os.path.join(os.path.dirname(__file__), '..', 'figures')
os.makedirs(FIG_DIR, exist_ok=True)


def read_csv(path):
    """Read CSV file, return list of dicts."""
    with open(path, 'r') as f:
        reader = csv.DictReader(f)
        return list(reader)


def plot_cpu_experiment():
    """Plot CPU usage vs food injection time series."""
    cpu_path = os.path.join(DATA_DIR, 'cpu_log.csv')
    if not os.path.exists(cpu_path):
        print(f"  Skipping CPU plot: {cpu_path} not found")
        return

    rows = read_csv(cpu_path)
    ticks = [int(r['tick']) for r in rows]
    cpu = [float(r['cpu_usage']) * 100 for r in rows]
    food = [int(r['food_injected']) for r in rows]

    if not HAS_MPL:
        print(f"  CPU data: {len(rows)} samples, avg CPU={sum(cpu)/len(cpu):.1f}%, avg food={sum(food)/len(food):.0f}")
        return

    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(12, 6), sharex=True)

    ax1.plot(ticks, cpu, color='red', alpha=0.7, linewidth=0.5)
    ax1.set_ylabel('CPU Usage (%)')
    ax1.set_title('Real CPU Experiment: System CPU Usage Over Time')
    ax1.set_ylim(0, 105)
    ax1.axhline(y=sum(cpu)/len(cpu), color='red', linestyle='--', alpha=0.5, label=f'Mean {sum(cpu)/len(cpu):.0f}%')
    ax1.legend()

    ax2.plot(ticks, food, color='green', alpha=0.7, linewidth=0.5)
    ax2.set_ylabel('Food Injected')
    ax2.set_xlabel('Tick')
    ax2.set_title('Food Injection Rate (inversely proportional to CPU usage)')
    ax2.axhline(y=sum(food)/len(food), color='green', linestyle='--', alpha=0.5, label=f'Mean {sum(food)/len(food):.0f}')
    ax2.legend()

    plt.tight_layout()
    out = os.path.join(FIG_DIR, 'cpu_food_timeseries.png')
    plt.savefig(out, dpi=150)
    plt.close()
    print(f"  Saved: {out}")


def plot_per_seed_distribution():
    """Plot experimental vs control REFRESH distribution from large-scale data."""
    path = os.path.join(DATA_DIR, 'large_scale', 'per_seed_summary.csv')
    if not os.path.exists(path):
        print(f"  Skipping distribution plot: {path} not found")
        return

    rows = read_csv(path)
    exp_refresh = [float(r['refresh_ratio']) * 100 for r in rows if r['group'] == 'exp']
    ctrl_refresh = [float(r['refresh_ratio']) * 100 for r in rows if r['group'] == 'ctrl']

    if not HAS_MPL:
        print(f"  100-seed data: exp REFRESH mean={sum(exp_refresh)/len(exp_refresh):.1f}%, "
              f"ctrl mean={sum(ctrl_refresh)/len(ctrl_refresh):.1f}%")
        return

    fig, axes = plt.subplots(1, 2, figsize=(12, 5))

    # Histogram
    axes[0].hist(exp_refresh, bins=20, alpha=0.7, color='blue', label='Experimental')
    axes[0].hist(ctrl_refresh, bins=20, alpha=0.7, color='orange', label='Control')
    axes[0].set_xlabel('REFRESH Ratio (%)')
    axes[0].set_ylabel('Count')
    axes[0].set_title('REFRESH Distribution (100 seeds)')
    axes[0].legend()

    # Box plot
    axes[1].boxplot([exp_refresh, ctrl_refresh], tick_labels=['Experimental', 'Control'])
    axes[1].set_ylabel('REFRESH Ratio (%)')
    axes[1].set_title('REFRESH: Exp vs Ctrl')

    plt.tight_layout()
    out = os.path.join(FIG_DIR, 'refresh_distribution_100seed.png')
    plt.savefig(out, dpi=150)
    plt.close()
    print(f"  Saved: {out}")


def plot_time_series():
    """Plot REFRESH% over time from time series data."""
    path = os.path.join(DATA_DIR, 'time_series_refresh.csv')
    if not os.path.exists(path):
        print(f"  Skipping time series plot: {path} not found")
        return

    rows = read_csv(path)
    ticks = [int(r['tick']) for r in rows]
    exp = [float(r['exp_refresh_ratio']) * 100 for r in rows]
    ctrl = [float(r['ctrl_refresh_ratio']) * 100 for r in rows]

    if not HAS_MPL:
        print(f"  Time series: {len(rows)} points")
        return

    fig, ax = plt.subplots(figsize=(12, 5))
    ax.plot(ticks, exp, color='blue', alpha=0.7, linewidth=0.8, label='Experimental')
    ax.plot(ticks, ctrl, color='orange', alpha=0.7, linewidth=0.8, label='Control')
    ax.set_xlabel('Tick')
    ax.set_ylabel('REFRESH Ratio (%)')
    ax.set_title('REFRESH Ratio Over Time (Seed 100)')
    ax.legend()

    plt.tight_layout()
    out = os.path.join(FIG_DIR, 'refresh_timeseries.png')
    plt.savefig(out, dpi=150)
    plt.close()
    print(f"  Saved: {out}")


if __name__ == '__main__':
    print("ExoMind Cell — Figure Generator\n")

    print("1. CPU experiment time series")
    plot_cpu_experiment()

    print("\n2. REFRESH distribution (100-seed)")
    plot_per_seed_distribution()

    print("\n3. REFRESH time series")
    plot_time_series()

    print(f"\nFigures saved to: {os.path.abspath(FIG_DIR)}")
