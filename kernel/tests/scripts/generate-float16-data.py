#!/usr/bin/env -S uv run --script
#
# /// script
# dependencies = [
#   "pyarrow==24.0.0",
#   "numpy==2.4.6",
# ]
# ///

"""
Generate the float16 test table at kernel/tests/data/float16/.

This script writes a small Delta table with a single Float16 column that uses
the `float16` table feature in its protocol.
"""

from __future__ import annotations

import argparse
import json
import time
import uuid
from pathlib import Path

import numpy as np
import pyarrow as pa
import pyarrow.parquet as pq


def write_parquet(out_dir: Path) -> tuple[str, int]:
    ids = pa.array([0, 1, 2, 3, 4], type=pa.int64())
    f16_vals = pa.array(
        [1.5, -2.25, 0.125, np.nan, None],
        type=pa.float16(),
    )
    table = pa.table({"id": ids, "f16": f16_vals})

    file_name = f"part-00000-4384565c-4672-4b77-8be6-2dddf3357aa4-c000.snappy.parquet"
    file_path = out_dir / file_name
    pq.write_table(table, file_path, compression="snappy")
    return file_name, file_path.stat().st_size


def write_log(log_dir: Path, file_name: str, file_size: int) -> None:
    log_dir.mkdir(parents=True, exist_ok=True)

    table_id = str(uuid.uuid4())
    now_ms = int(time.time() * 1000)
    schema_string = json.dumps(
        {
            "type": "struct",
            "fields": [
                {
                    "name": "id",
                    "type": "long",
                    "nullable": True,
                    "metadata": {},
                },
                {
                    "name": "f16",
                    "type": "float16",
                    "nullable": True,
                    "metadata": {},
                },
            ],
        }
    )

    stats = json.dumps(
        {
            "numRecords": 5,
            "minValues": {"id": 0, "f16": -2.25},
            "maxValues": {"id": 4, "f16": 1.5},
            "nullCount": {"id": 0, "f16": 1},
        }
    )

    commit_info = {
        "commitInfo": {
            "timestamp": now_ms,
            "operation": "WRITE",
            "operationParameters": {"mode": "ErrorIfExists"},
            "engineInfo": "generate-float16-data.py",
            "clientVersion": "generate-float16-data.py-1.0",
            "operationMetrics": {
                "execution_time_ms": 0,
                "num_added_files": 1,
                "num_added_rows": 3,
                "num_partitions": 0,
                "num_removed_files": 0,
            },
        }
    }
    protocol = {
        "protocol": {
            "minReaderVersion": 3,
            "minWriterVersion": 7,
            "readerFeatures": ["float16"],
            "writerFeatures": ["float16"],
        }
    }
    metadata = {
        "metaData": {
            "id": table_id,
            "name": None,
            "description": None,
            "format": {"provider": "parquet", "options": {}},
            "schemaString": schema_string,
            "partitionColumns": [],
            "createdTime": now_ms,
            "configuration": {},
        }
    }
    add = {
        "add": {
            "path": file_name,
            "partitionValues": {},
            "size": file_size,
            "modificationTime": now_ms,
            "dataChange": True,
            "stats": stats,
            "tags": None,
            "baseRowId": None,
            "defaultRowCommitVersion": None,
            "clusteringProvider": None,
        }
    }

    commit_path = log_dir / "00000000000000000000.json"
    with commit_path.open("w") as f:
        for action in (commit_info, protocol, metadata, add):
            f.write(json.dumps(action))
            f.write("\n")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    default_out = (
        Path(__file__).resolve().parent.parent / "data" / "float16"
    )
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=default_out,
        help=f"Directory to write the table into (default: {default_out})",
    )
    args = parser.parse_args()

    out_dir: Path = args.out_dir
    out_dir.mkdir(parents=True, exist_ok=True)
    log_dir = out_dir / "_delta_log"

    file_name, file_size = write_parquet(out_dir)
    write_log(log_dir, file_name, file_size)
    print(f"Wrote table to {out_dir}")


if __name__ == "__main__":
    main()
