import typing
import shlex
import json
import subprocess
import pathlib
from collections import defaultdict
import attrs
import cattrs

TARGET = str(pathlib.Path("./target/release/experiment").absolute())


class ExperimentArgs(typing.Protocol):
    name: str

    def to_cmd(self) -> list[str]:
        ...


@attrs.define
class Mphf:
    positive_keys: int
    total_keys: int
    kmer_size: int
    name: str = attrs.field(default="mphf", init=False)

    def to_cmd(self):
        return shlex.split(
            f"{TARGET} -t {self.total_keys} -p {self.positive_keys} -k {self.kmer_size} mphf"
        )


@attrs.define
class BloomFilter:
    positive_keys: int
    total_keys: int
    kmer_size: int
    fpp: float
    name: str = attrs.field(init=False, default="bloom-filter")

    def to_cmd(self):
        return shlex.split(
            f"{TARGET} -t {self.total_keys} -p {self.positive_keys} -k {self.kmer_size} {self.name} --fpp {self.fpp:.12f}"
        )


@attrs.define
class Fingerprint:
    positive_keys: int
    total_keys: int
    kmer_size: int
    width: int
    name: str = attrs.field(init=False, default="fingerprint")

    def to_cmd(self):
        return shlex.split(
            f"{TARGET} -t {self.total_keys} -p {self.positive_keys} -k {self.kmer_size} {self.name} --width {self.width}"
        )


def execute_run(arg: ExperimentArgs) -> dict:
    result = subprocess.run(arg.to_cmd(), capture_output=True, check=True, text=True)
    return {**cattrs.unstructure(arg), **json.loads(result.stdout)}


def main(args: list[ExperimentArgs], outfile=None) -> None:
    target_dir = pathlib.Path(TARGET)
    if not target_dir.exists():
        subprocess.run(shlex.split("cargo build --release"))

    outfile = pathlib.Path(outfile if outfile is not None else "results.json")
    results = defaultdict(list)
    for idx, arg in enumerate(args, start=1):
        print(f"Completed experiment {idx: 3}/{len(args)}")
        results[arg.name].append(execute_run(arg))
    outfile.write_text(json.dumps(results, indent=4))


if __name__ == "__main__":
    args: list[ExperimentArgs] = []
    kmer_size = 30
    runs = 5
    for run in range(runs):
        for total_keys in [10_000, 50_000, 100_000]:
            for pct in map(lambda x: x / 10, range(1, 10)):
                positive_keys = int(total_keys * pct)
                args.append(
                    Mphf(
                        positive_keys=positive_keys,
                        total_keys=total_keys,
                        kmer_size=kmer_size,
                    )
                )
                for fpp in [1 / (2**x) for x in [7, 8, 10]]:
                    args.append(
                        BloomFilter(
                            positive_keys=positive_keys,
                            total_keys=total_keys,
                            kmer_size=kmer_size,
                            fpp=fpp,
                        )
                    )
                for width in [7, 8, 10]:
                    args.append(
                        Fingerprint(
                            positive_keys=positive_keys,
                            total_keys=total_keys,
                            kmer_size=kmer_size,
                            width=width,
                        )
                    )
    main(args, outfile="results.json")
