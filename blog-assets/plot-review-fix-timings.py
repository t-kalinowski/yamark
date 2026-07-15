from __future__ import annotations

import csv
import math
import statistics
from pathlib import Path

import matplotlib

matplotlib.use("Agg")

import matplotlib.pyplot as plt
from matplotlib.ticker import FixedLocator, FuncFormatter


HERE = Path(__file__).parent
DATA_PATH = HERE / "review-fix-timings.csv"
OUTPUT_PATH = HERE / "review-fix-timings.png"

REVIEW_COLOR = "#146C94"
FIX_COLOR = "#D97706"
MUTED_COLOR = "#6B7280"


def rolling_median(values: list[float], window: int) -> tuple[list[int], list[float]]:
    stop_rounds: list[int] = []
    medians: list[float] = []
    for stop in range(window, len(values) + 1):
        sample = [value for value in values[stop - window : stop] if math.isfinite(value)]
        stop_rounds.append(stop)
        medians.append(statistics.median(sample))
    return stop_rounds, medians


with DATA_PATH.open(newline="", encoding="utf-8") as csv_file:
    rows = list(csv.DictReader(csv_file))

paired = [row for row in rows if row["row_type"] == "matched_pair"]
terminal = next(row for row in rows if row["row_type"] == "terminal_review")
rounds = [int(row["pair_index"]) for row in paired]
review_minutes = [float(row["review_wall_seconds"]) / 60 for row in paired]
fix_minutes = [
    float(row["fix_wall_seconds"]) / 60 if row["fix_status"] == "complete" else math.nan
    for row in paired
]
aborted_rounds = [
    int(row["pair_index"]) for row in paired if row["fix_status"] == "turn_aborted"
]
aborted_minutes = [
    float(row["fix_wall_seconds"]) / 60
    for row in paired
    if row["fix_status"] == "turn_aborted"
]
terminal_round = int(terminal["plot_index"])
terminal_minutes = float(terminal["review_wall_seconds"]) / 60

review_stops, review_medians = rolling_median(review_minutes, window=10)
fix_stops, fix_medians = rolling_median(fix_minutes, window=10)

plt.rcParams.update(
    {
        "font.family": "sans-serif",
        "axes.titleweight": "bold",
        "axes.edgecolor": "#9CA3AF",
        "axes.labelcolor": "#1F2937",
        "text.color": "#111827",
        "xtick.color": "#4B5563",
        "ytick.color": "#4B5563",
    }
)

fig, ax = plt.subplots(figsize=(11.5, 6.5), dpi=180)
fig.patch.set_facecolor("white")
ax.set_facecolor("#FAFAFA")

ax.scatter(
    rounds,
    review_minutes,
    color=REVIEW_COLOR,
    alpha=0.35,
    marker="o",
    s=20,
    label="Review, matched pair",
)
ax.scatter(
    rounds,
    fix_minutes,
    color=FIX_COLOR,
    alpha=0.35,
    marker="^",
    s=22,
    label="Fix, completed matched pair",
)
ax.plot(
    review_stops,
    review_medians,
    color=REVIEW_COLOR,
    linewidth=3,
    label="Review, 10-pair median",
)
ax.plot(
    fix_stops,
    fix_medians,
    color=FIX_COLOR,
    linewidth=3,
    linestyle="--",
    label="Fix, 10-pair median",
)
ax.scatter(
    aborted_rounds,
    aborted_minutes,
    color=MUTED_COLOR,
    marker="x",
    s=38,
    linewidth=1.5,
    zorder=5,
    label="Aborted fix",
)
ax.scatter(
    [terminal_round],
    [terminal_minutes],
    color=REVIEW_COLOR,
    marker="*",
    s=135,
    zorder=6,
    label="Terminal review",
)

ax.axvline(20.5, color="#9CA3AF", linestyle="--", linewidth=1)
ax.text(
    21.1,
    2.2,
    "largest gap: six days",
    rotation=90,
    va="bottom",
    ha="left",
    fontsize=9,
    color=MUTED_COLOR,
)

ax.annotate(
    "terminal review\n6h 52m",
    xy=(terminal_round, terminal_minutes),
    xytext=(66, 250),
    arrowprops={"arrowstyle": "-", "color": REVIEW_COLOR, "linewidth": 1.2},
    color=REVIEW_COLOR,
    fontsize=10,
    ha="right",
)
ax.annotate(
    "final successful fix\n2m 49s",
    xy=(rounds[-1], fix_minutes[-1]),
    xytext=(67, 2.2),
    arrowprops={"arrowstyle": "-", "color": FIX_COLOR, "linewidth": 1.2},
    color=FIX_COLOR,
    fontsize=10,
    ha="right",
)

ax.set_title("Yamark V2 review and fix session duration", loc="left", fontsize=18, pad=18)
ax.text(
    0,
    1.02,
    "73 matched review→fix attempts plus the terminal review; unmatched calls omitted · May 21–30, 2026 (ET)",
    transform=ax.transAxes,
    fontsize=10.5,
    color=MUTED_COLOR,
)
ax.set_xlabel("Chronological matched-pair index")
ax.set_ylabel("Wall time (minutes, log scale)")
ax.set_xlim(0, 77)
ax.set_yscale("log")
ax.set_ylim(1.8, 600)
ticks = [2, 5, 10, 20, 60, 120, 240, 480]
ax.yaxis.set_major_locator(FixedLocator(ticks))
ax.yaxis.set_major_formatter(FuncFormatter(lambda value, _: f"{int(value)}"))
ax.grid(axis="y", which="major", color="#D1D5DB", linewidth=0.8)
ax.grid(axis="y", which="minor", visible=False)
ax.spines[["top", "right"]].set_visible(False)

handles, labels = ax.get_legend_handles_labels()
order = [2, 3, 0, 1, 4, 5]
ax.legend(
    [handles[index] for index in order],
    [labels[index] for index in order],
    loc="upper left",
    ncols=2,
    frameon=False,
    fontsize=9,
)

fig.text(
    0.075,
    0.012,
    "Wall time runs from session start to final event. Raw points are unconnected; rolling fix medians exclude two aborted sessions.",
    fontsize=8.5,
    color=MUTED_COLOR,
)
fig.tight_layout(rect=(0.02, 0.04, 0.99, 0.98))
fig.savefig(OUTPUT_PATH, bbox_inches="tight")
