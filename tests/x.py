#!/usr/bin/env python
import subprocess
import os
import sys
import time
import argparse
from glob import glob

parser = argparse.ArgumentParser()
parser.add_argument("-t", "--target")
parser.add_argument("-l", "--list-targets", action="store_true")
parser.add_argument("--test")

argv, unknown = parser.parse_known_args()


def start(service):
    res = subprocess.run(
        ["docker-compose", "up", "-d", service],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        cwd=os.path.dirname(__file__),
    )

    if res.returncode != 0:
        print(res.stderr, file=sys.stderr)

    if b"done" in res.stderr:
        time.sleep(30)


def run(command, comment=None, env=None, service=None, tag=None, args=None):
    if argv.list_targets:
        if tag:
            print(f"{tag}")

        return

    if argv.target and tag != argv.target:
        return

    if comment is not None:
        print(f"\x1b[2m # {comment}\x1b[0m")

    environ = []
    if env is not None:
        for name, value in env.items():
            print(f"\x1b[93m $ {name}={value}\x1b[0m")
            environ.append(f"-e{name}={value}")

    if service is not None:
        start(service)

    command_args = []

    if argv.test:
        command_args.extend(["--test", argv.test])

    if unknown:
        command_args.extend(["--", *unknown])

        if args is not None:
            command_args.extend(args)

    print(f"\x1b[93m $ {command} {' '.join(command_args)}\x1b[0m")

    res = subprocess.run(
        [
            "docker-compose",
            "run",
            "--user",
            f"{os.getuid()}:{os.getgid()}",
            "--rm",
            *environ,
            "sqlx",
            *command.split(" "),
            *command_args
        ],
        cwd=os.path.dirname(__file__),
    )

    if res.returncode != 0:
        sys.exit(res.returncode)


# before we start, we clean previous profile data
# keeping these around can cause weird errors
for path in glob(os.path.join(os.path.dirname(__file__), "target/**/*.gc*"), recursive=True):
    os.remove(path)

#
# check
#

run("cargo c", comment="check with a default set of features", tag="check")

run(
    "cargo c --no-default-features --features runtime-async-std,all-databases,all-types",
    comment="check with async-std",
    tag="check_async_std"
)

run(
    "cargo c --no-default-features --features runtime-tokio,all-databases,all-types",
    comment="check with tokio",
    tag="check_tokio"
)

run(
    "cargo c --no-default-features --features runtime-actix,all-databases,all-types",
    comment="check with actix",
    tag="check_actix"
)

#
# unit test
#

run(
    "cargo test --manifest-path sqlx-core/Cargo.toml --features mysql,postgres,sqlite,all-types",
    comment="unit test core",
    tag="unit"
)

run(
    "cargo test --no-default-features --manifest-path sqlx-core/Cargo.toml --features mysql,postgres,sqlite,all-types,runtime-tokio",
    comment="unit test core",
    tag="unit_tokio"
)

#
# integration tests
#

for runtime in ["async-std", "tokio", "actix"]:

    #
    # sqlite
    #

    run(
        f"cargo test --no-default-features --features all-types,sqlite,runtime-{runtime}",
        comment=f"test sqlite",
        env={"DATABASE_URL": f"sqlite://tests/sqlite/sqlite.db"},
        tag=f"sqlite" if runtime == "async-std" else f"sqlite_{runtime}",
        # FIXME: The SQLite driver does not currently support concurrent access to the same database
        args=["--test-threads=1"],
    )

    #
    # postgres
    #

    for version in ["12", "10", "9.6", "9.5"]:
        v = version.replace(".", "_")
        run(
            f"cargo test --no-default-features --features all-types,postgres,runtime-{runtime}",
            comment=f"test postgres {version}",
            env={"DATABASE_URL": f"postgres://postgres:password@postgres_{v}/sqlx"},
            service=f"postgres_{v}",
            tag=f"postgres_{v}" if runtime == "async-std" else f"postgres_{v}_{runtime}",
        )

    #
    # postgres ssl
    #

    for version in ["12", "10", "9.6", "9.5"]:
        v = version.replace(".", "_")
        run(
            f"cargo test --no-default-features --features all-types,postgres,runtime-{runtime}",
            comment=f"test postgres {version} ssl",
            env={
                "DATABASE_URL": f"postgres://postgres:password@postgres_{v}/sqlx?sslmode=verify-ca&sslrootcert=.%2Ftests%2Fcerts%2Fca.crt"
            },
            service=f"postgres_{v}",
            tag=f"postgres_{v}_ssl" if runtime == "async-std" else f"postgres_{v}_ssl_{runtime}",
        )

    #
    # mysql
    #

    for version in ["8", "5.7", "5.6"]:
        v = version.replace(".", "_")
        run(
            f"cargo test --no-default-features --features all-types,mysql,runtime-{runtime}",
            comment=f"test mysql {version}",
            env={"DATABASE_URL": f"mysql://root:password@mysql_{v}/sqlx"},
            service=f"mysql_{v}",
            tag=f"mysql_{v}" if runtime == "async-std" else f"mysql_{v}_{runtime}",
        )

    #
    # mariadb
    #

    for version in ["10_5", "10_4", "10_3", "10_2", "10_1"]:
        v = version.replace(".", "_")
        run(
            f"cargo test --no-default-features --features all-types,mysql,runtime-{runtime}",
            comment=f"test mariadb {version}",
            env={"DATABASE_URL": f"mysql://root:password@mariadb_{v}/sqlx"},
            service=f"mariadb_{v}",
            tag=f"mariadb_{v}" if runtime == "async-std" else f"mariadb_{v}_{runtime}",
        )

# TODO: Use [grcov] if available
# ~/.cargo/bin/grcov tests/.cache/target/debug -s sqlx-core/ -t html --llvm --branch -o ./target/debug/coverage
