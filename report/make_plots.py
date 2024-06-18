import matplotlib.pyplot as plt
import matplotlib.ticker as ticker
import pandas as pd

df = pd.read_csv("../performance_test/res.csv", sep=";", header="infer")
suptitle_fontsize = 16
figsize = (6, 4)
dpi = 500

grouped_dfs = {key: group for key, group in df.groupby("figure_name")}
for k, d in grouped_dfs.items():
    d.dropna(inplace=True, axis=1, how="all")
    d.drop(inplace=True, columns="figure_name")


def print_scatter(title, filename, df, ylabel, xlabel):
    fig, ax = plt.subplots(1, 1)
    df = df.melt(id_vars="series_name")
    df["series_name"] = df["series_name"].astype("int64")
    df = df.drop(columns="variable")
    fig, ax = plt.subplots(1, 1, figsize=figsize, dpi=dpi)
    ax.scatter(df["series_name"], df["value"], marker=".", color="green")
    ax.set_ylabel(ylabel)
    ax.set_xlabel(xlabel)
    ax.set_yscale("log")
    ax.set_xscale("log", base=2)
    ax.grid(axis="y")
    fig.suptitle(title, fontsize=suptitle_fontsize)
    fig.tight_layout()
    fig.savefig(f"assets/{filename}.png")


print_scatter(
    "Time to Write a File Across Sizes",
    "file_write",
    grouped_dfs["file_write"],
    "Time for 10000 Iterations (s)",
    "File Size (Bytes)",
)

print_scatter(
    r"Runtime of $\mathtt{base64}$ Across Input File Sizes",
    "dynamic_file_stdin",
    grouped_dfs["dynamic_file_stdin"],
    "Time for 10000 Iterations (s)",
    "Input File Size (Bytes)",
)

print_scatter(
    "Time to Use a Shared Memory Segment Across Sizes",
    "shmem",
    grouped_dfs["simple_shmem_persist_fill_check"],
    "Time for 10000 Iterations (s)",
    "Shared Memory Size (Bytes)",
)


def print_boxplot(title, filename, df, ylabel):
    fig, ax = plt.subplots(1, 1)
    df = df.transpose()
    df.columns = df.iloc[0]
    df = df[1:].reset_index(drop=True)
    df = df.astype("float64")
    fig, ax = plt.subplots(1, 1, figsize=figsize, dpi=dpi)
    df.boxplot(column=list(df.columns), ax=ax, color="green")
    ax.set_ylabel(ylabel)
    ax.grid(axis="y")
    fig.suptitle(title, fontsize=suptitle_fontsize)
    fig.tight_layout()
    fig.savefig(f"assets/{filename}.png")


print_boxplot(
    r"Performance of $\mathtt{stdin}$ Options",
    "stdin_types",
    grouped_dfs["stdin_types"],
    "Time for 10000 Iterations (s)",
)

print_boxplot(
    r"Performance of $\mathtt{LD\_PRELOAD}$ Options",
    "preloads",
    grouped_dfs["preloads"],
    "Time for 10000 Iterations (s)",
)


def print_boxplots_instrumentation(title, filename, ax_titles, dfs, ylabel):
    fig, axes = plt.subplots(1, len(dfs), figsize=figsize, dpi=dpi)
    axes[0].set_ylabel(ylabel)
    for ax, df, ax_title in zip(axes, dfs, ax_titles):
        df = df.transpose()
        df.columns = df.iloc[0]
        df = df[1:].reset_index(drop=True)
        df = df.astype("float64")
        df.boxplot(column=list(df.columns), ax=ax, color="green")
        # plt.setp(ax.get_xticklabels(), rotation=45)
        # ax.semilogy()
        ax.grid(axis="y")
        ax.set_title(ax_title)
    fig.suptitle(title, fontsize=suptitle_fontsize)
    fig.tight_layout()
    fig.savefig(f"assets/{filename}.png")


print_boxplots_instrumentation(
    r"Performance of Coverage Instrumentation",
    "instrumentation",
    ["No Instrumentation", "GNU", "uutils"],
    [
        grouped_dfs["instrumentation_no"],
        grouped_dfs["instrumentation_GNU"],
        grouped_dfs["instrumentation_uutils"],
    ],
    "Time for 10000 Iterations (s)",
)
