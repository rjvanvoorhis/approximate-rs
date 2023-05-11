import typing
import json
import pathlib
import datetime
import functools
from fractions import Fraction
from matplotlib import pyplot as plt
import pandas as pd
import numpy as np

try:
    from collections.abc import Mapping
except ImportError:
    from collections import Mapping


class ExperimentResult(typing.TypedDict):
    kmer_size: int
    total_keys: int
    positive_keys: int
    negative_keys: int
    serialized_size: int
    false_positive_count: int
    false_negative_count: int
    negatives_query_duration: datetime.timedelta
    positives_query_duration: datetime.timedelta
    name: str
    width: typing.Optional[int]
    fpp: typing.Optional[float]


class Results(typing.TypedDict):
    mphf: list[ExperimentResult]
    fingerprint: list[ExperimentResult]
    bloom_filter: list[ExperimentResult]


def compose(
    *funcs: typing.Callable[[typing.Any], typing.Any]
) -> typing.Callable[[typing.Any], typing.Any]:
    return functools.reduce(
        lambda left, right: lambda x: left(right(x)), reversed(funcs)
    )


def read_data(filename: str) -> dict:
    return json.loads(pathlib.Path(filename).read_text())


def is_duration(obj: typing.Any) -> bool:
    return isinstance(obj, dict) and all(key in obj for key in ["secs", "nanos"])


def convert_keys(obj: dict) -> dict:
    return {k.replace("-", "_"): v for k, v in obj.items()}


def convert_timedelta(data: typing.Any) -> typing.Any:
    if isinstance(data, Mapping):
        if all(key in data for key in ["secs", "nanos"]):
            return datetime.timedelta(
                seconds=data["secs"], microseconds=data["nanos"] * 0.001
            )
        return {k: convert_timedelta(v) for k, v in data.items()}
    if isinstance(data, (list, tuple, set)):
        return type(data)(map(convert_timedelta, data))
    return data


def enhance(df: pd.DataFrame) -> pd.DataFrame:
    df["observed_fpp"] = df["false_positive_count"] / (df["negative_keys"])
    df["ratio"] = df["positive_keys"] / df["total_keys"]
    df["avg_positive_query_duration"] = (
        df["positives_query_duration"] / df["positive_keys"]
    )
    df["avg_negative_query_duration"] = (
        df["negatives_query_duration"] / df["negative_keys"]
    )
    df["avg_query_nanoseconds"] = (
        (df["positives_query_duration"] + df["negatives_query_duration"])
        / df["total_keys"]
    ).map(lambda x: x.nanoseconds)
    df["bits_per_key"] = (df["serialized_size"] * 8) / df["positive_keys"]
    return df


def plot_task_1(outdir: str, bloom_filter_results: list[ExperimentResult]):
    folder = pathlib.Path(outdir)
    folder.mkdir(exist_ok=True)
    df = pd.DataFrame(bloom_filter_results)
    df["fpp_fraction"] = df["fpp"].map(Fraction)
    df = enhance(df)
    x_label = "Target False Positive Rate"
    for group, sub_df in df.groupby("total_keys"):
        pivot_table = pd.pivot_table(
            sub_df,
            values="observed_fpp",
            index="fpp_fraction",
            columns="positive_keys",
            aggfunc=np.mean,
        )
        title = f"Bloom Filter With {group} Total Keys"
        plot_pivot_table(
            pivot_table,
            filename=str(folder / f"bloom-filter-{group}-expected-vs-observed-fpp.png"),
            title=title,
            x_label=x_label,
            y_label="Observed False Positive Rate",
        )

        pivot_table = pd.pivot_table(
            sub_df,
            values="avg_query_nanoseconds",
            index="fpp_fraction",
            columns="positive_keys",
            aggfunc=np.mean,
            fill_value=datetime.timedelta(microseconds=0),
        )
        plot_pivot_table(
            pivot_table,
            filename=str(folder / f"bloom-filter-{group}-avg-query-duration.png"),
            title=title,
            x_label=x_label,
            y_label="Average Query Duration (ns)",
            move_legend=True,
        )

        pivot_table = pd.pivot_table(
            sub_df,
            values="serialized_size",
            index="fpp_fraction",
            columns="positive_keys",
            aggfunc=np.mean,
            fill_value=datetime.timedelta(microseconds=0),
        )
        plot_pivot_table(
            pivot_table,
            filename=str(folder / f"bloom-filter-{group}-size.png"),
            title=title,
            x_label=x_label,
            y_label="Datastructure Size (bytes)",
            move_legend=True,
        )
        pivot_table = pd.pivot_table(
            sub_df,
            values="bits_per_key",
            index="fpp_fraction",
            columns="positive_keys",
            aggfunc=np.mean,
            fill_value=datetime.timedelta(microseconds=0),
        )
        plot_pivot_table(
            pivot_table,
            filename=str(folder / f"bloom-filter-{group}-bits-per-key.png"),
            title=title,
            x_label=x_label,
            y_label="Bits per key",
            move_legend=True,
        )


def plot_task_2(outdir: str, mphf_results: list[ExperimentResult]):
    folder = pathlib.Path(outdir)
    folder.mkdir(exist_ok=True)
    df = pd.DataFrame(mphf_results)
    df = enhance(df)
    df = (
        df.groupby(["total_keys", "positive_keys"])
        .mean(numeric_only=True)
        .reset_index()
    )
    x_label = "Positive Keys"
    for group, sub_df in df.groupby("total_keys"):
        ax = sub_df.plot(x="positive_keys", y="observed_fpp", kind="bar")
        ax.set_title(f"MPHF With {group} Total Keys")
        ax.set_xlabel(x_label)
        ax.set_ylabel("Observed False Positive Rate")
        ax.set_xticklabels(ax.get_xticklabels(), rotation=45)
        ax.get_legend().remove()
        fig = ax.get_figure()
        fig.savefig(
            str(folder / f"mphf-{group}-observed-false-positive-rate.png"),
            bbox_inches="tight",
        )

        ax = sub_df.plot(x="positive_keys", y="avg_query_nanoseconds", kind="bar")
        ax.set_title(f"MPHF With {group} Total Keys")
        ax.set_xlabel(x_label)
        ax.set_ylabel("Average Query Duration (ns)")
        ax.set_xticklabels(ax.get_xticklabels(), rotation=45)
        ax.get_legend().remove()
        fig = ax.get_figure()
        fig.savefig(
            str(folder / f"mphf-{group}-query-duration.png"), bbox_inches="tight"
        )

        ax = sub_df.plot(x="positive_keys", y="serialized_size", kind="bar")
        ax.set_title(f"MPHF With {group} Total Keys")
        ax.set_xlabel(x_label)
        ax.set_ylabel("Datastructure Size (bytes)")
        ax.set_xticklabels(ax.get_xticklabels(), rotation=45)
        ax.get_legend().remove()
        fig = ax.get_figure()
        fig.savefig(str(folder / f"mphf-{group}-size.png"), bbox_inches="tight")

        ax = sub_df.plot(x="positive_keys", y="bits_per_key", kind="bar")
        ax.set_title(f"MPHF With {group} Total Keys")
        ax.set_xlabel(x_label)
        ax.set_ylabel("bits per key")
        ax.set_xticklabels(ax.get_xticklabels(), rotation=45)
        ax.get_legend().remove()
        fig = ax.get_figure()
        fig.savefig(str(folder / f"mphf-{group}-bits-per-key.png"), bbox_inches="tight")


def plot_task_3(outdir: str, bloom_filter_results: list[ExperimentResult]):
    folder = pathlib.Path(outdir)
    folder.mkdir(exist_ok=True)
    df = pd.DataFrame(bloom_filter_results)
    df = enhance(df)

    x_label = "Fingerprint Width (bits)"
    for group, sub_df in df.groupby("total_keys"):
        pivot_table = pd.pivot_table(
            sub_df,
            values="observed_fpp",
            index="width",
            columns="positive_keys",
            aggfunc=np.mean,
        )
        title = f"Fingerprint Array with {group} Total Keys"
        plot_pivot_table(
            pivot_table=pivot_table,
            filename=str(
                folder / f"fingerprint-array-{group}-width-vs-observed-fpp.png"
            ),
            title=title,
            x_label=x_label,
            y_label="Observed False Positive Rate",
            rotation=0,
            move_legend=True,
        )

        pivot_table = pd.pivot_table(
            sub_df,
            values="avg_query_nanoseconds",
            index="width",
            columns="positive_keys",
            aggfunc=np.mean,
        )
        plot_pivot_table(
            pivot_table=pivot_table,
            filename=str(folder / f"fingerprint-array-{group}-avg-query-duration.png"),
            title=title,
            x_label=x_label,
            y_label="Observed False Positive Rate",
            rotation=0,
            move_legend=True,
        )

        pivot_table = pd.pivot_table(
            sub_df,
            values="serialized_size",
            index="width",
            columns="positive_keys",
            aggfunc=np.mean,
        )
        plot_pivot_table(
            pivot_table,
            filename=str(folder / f"fingerprint-array-{group}-size.png"),
            title=f"Fingerprint Array With {group} Total Keys",
            x_label=x_label,
            y_label=f"Datastructure Size (bytes)",
            move_legend=True,
        )

        pivot_table = pd.pivot_table(
            sub_df,
            values="bits_per_key",
            index="width",
            columns="positive_keys",
            aggfunc=np.mean,
        )
        plot_pivot_table(
            pivot_table,
            filename=str(folder / f"fingerprint-array-{group}-bits-per-key.png"),
            title=f"Fingerprint Array With {group} Total Keys",
            x_label=x_label,
            y_label=f"Bits per key",
            move_legend=True,
        )


def plot_pivot_table(
    pivot_table: pd.DataFrame,
    filename: str,
    title: str,
    x_label: str,
    y_label: str,
    rotation: int = 0,
    remove_legend: bool = False,
    move_legend: bool = False,
):
    ax = pivot_table.plot(kind="bar")
    ax.set_title(title)
    ax.set_xlabel(x_label)
    ax.set_ylabel(y_label)
    ax.set_xticklabels(ax.get_xticklabels(), rotation=rotation)
    if move_legend:
        pos = ax.get_position()
        ax.set_position([pos.x0, pos.y0, pos.width * 0.9, pos.height])
        ax.legend(title="positive keys", loc="center right", bbox_to_anchor=(1.25, 0.5))
    if remove_legend:
        ax.get_legend().remove()
    fig = ax.get_figure()
    fig.savefig(filename, bbox_inches="tight")
    plt.close()


if __name__ == "__main__":
    import time

    start = time.time()
    results: Results = compose(read_data, convert_keys, convert_timedelta)(
        "results.json"
    )
    df = pd.DataFrame(results["fingerprint"])
    plot_task_1("results", results["bloom_filter"])
    plot_task_2("results", results["mphf"])
    plot_task_3("results", results["fingerprint"])
