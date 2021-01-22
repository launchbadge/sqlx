#!/usr/bin/env python3
import argparse
import subprocess
import re
import sys
from os import environ
from pathlib import Path
from shutil import rmtree
from subprocess import check_call, check_output, PIPE

parser = argparse.ArgumentParser()
parser.add_argument("-v", "--verbose", action="store_true")
parser.add_argument("-q", "--quiet", action="store_true")
parser.add_argument("-t", "--target")
parser.add_argument("-l", "--list-targets", action="store_true")
parser.add_argument("--coverage", action="store_true")

argv, unknown = parser.parse_known_args()

# get an absolute path to the project directory for SQLx
project_dir = Path(__file__).parent
coverage_dir = project_dir.joinpath(".coverage")
raw_coverage_dir = coverage_dir.joinpath("raw")

# global test filenames
# we capture these all, so we can collect coverage later
test_object_filenames = []


def should_run(tag):
    if argv.list_targets:
        print(tag)
        return False

    if argv.target is not None:
        if not re.match(argv.target.replace("_", ".*?"), tag):
            return False

    return True


def run(cmd, *, cwd, env=None, comment=None, tag=None):
    if comment is not None:
        print(f"\x1b[2m # {comment} [{tag}]\x1b[0m")

    if not argv.quiet:
        print(f"\x1b[93m $ {' '.join(cmd)}\x1b[0m")

    res = subprocess.run(cmd, env=env, cwd=project_dir, check=False, stdout=None if argv.verbose else PIPE)

    if not argv.verbose and res.returncode != 0:
        print(res.stdout.decode())

    if res.returncode != 0:
        sys.exit(1)


def run_checks(project: str, *, cmd="check"):
    run_check(project, args=[], cmd=cmd)
    run_check(project, args=["--features", "blocking"], variant="blocking", cmd=cmd)
    run_check(project, args=["--features", "async"], variant="async", cmd=cmd)
    run_check(project, args=["--all-features"], variant="all", cmd=cmd)


def run_check(project: str, *, args, variant=None, cmd="check"):
    comment = f"{cmd} {project}"
    tag = f"{cmd}:{project.rsplit('-', maxsplit=1)[-1]}"

    if variant is not None:
        tag += f":{variant}"
        comment += f" +{variant}"

    if not should_run(tag):
        return

    # update timestamp to ensure check runs
    Path(f"{project}/src/lib.rs").touch()

    run([x for x in [
        "cargo", "+nightly", cmd,
        "-q" if argv.quiet else None,
        "--message-format", "human" if argv.verbose else "short",
        "--manifest-path", f"{project}/Cargo.toml",
        *args,
    ] if x], cwd=project_dir, comment=comment, tag=tag)


def run_docs(project: str):
    comment = f"doc {project}"
    tag = f"doc:{project.rsplit('-', maxsplit=1)[-1]}"

    if should_run(tag):
        env = environ.copy()
        env["RUSTDOCFLAGS"] = "--cfg doc_cfg"

        run([x for x in [
            "cargo", "+nightly", "doc",
            "-q" if argv.quiet else None,
            "--manifest-path", f"{project}/Cargo.toml",
            "--all-features",
        ] if x], cwd=project_dir, comment=comment, tag=tag, env=env)


def run_unit_test(project: str):
    tag = f"unit:{project.rsplit('-', maxsplit=1)[-1]}"

    if not should_run(tag):
        return

    env = environ.copy()
    env["LLVM_PROFILE_FILE"] = f"{project_dir}/.coverage/raw/{project}_%m.profraw"

    if argv.coverage:
        env["RUSTFLAGS"] = "-Zinstrument-coverage"

    # run the tests
    run([x for x in [
        "cargo", "+nightly", "test",
        "-q" if argv.quiet else None,
        "--manifest-path", f"{project}/Cargo.toml",
        "--features", "async",
    ] if x], env=env, cwd=project_dir, comment=f"unit test {project}", tag=tag)

    # build the test binaries and outputs a big pile of JSON that can help
    # us figure out the test binary filename (needed for coverage results)
    messages = subprocess.run([
        "cargo", "+nightly", "test",
        "--manifest-path", f"{project}/Cargo.toml",
        "--features", "async",
        "--no-run", "--message-format=json"
    ], env=env, cwd=project_dir, check=True, capture_output=True).stdout

    # use jq to extract the test filenames from the json blob
    # TODO: use python json to remove the dependency on jq
    filenames = check_output([
        "jq", "-r", "select(.profile.test == true) | .filenames[]",
    ], env=env, cwd=project_dir, input=messages).decode().strip().split("\n")

    test_object_filenames.extend(filter(lambda fn: not fn.endswith(".dSYM"), filenames))


def main():
    # remove and re-create directory for raw coverage data
    coverage_dir.mkdir(parents=True, exist_ok=True)
    rmtree(raw_coverage_dir, ignore_errors=True)
    raw_coverage_dir.mkdir(parents=True)

    # run checks
    run_checks("sqlx-core")
    run_checks("sqlx-mysql")
    run_checks("sqlx-postgres")
    run_checks("sqlx")

    # run checks
    run_checks("sqlx-core", cmd="clippy")
    run_checks("sqlx-mysql", cmd="clippy")
    run_checks("sqlx-postgres", cmd="clippy")
    run_checks("sqlx", cmd="clippy")

    # run docs (only if asked)
    run_docs("sqlx-core")
    run_docs("sqlx-mysql")
    run_docs("sqlx-postgres")
    run_docs("sqlx")

    # run unit tests, collect test binary filenames
    run_unit_test("sqlx-core")
    run_unit_test("sqlx-mysql")
    run_unit_test("sqlx")

    if test_object_filenames and argv.coverage:
        # merge raw profile data into a single profile
        check_call([
            "cargo", "profdata", "--", "merge",
            "--sparse",
            "-o", f"{project_dir}/.coverage/sqlx.profdata",
            *raw_coverage_dir.glob("*.profraw"),
        ], cwd=project_dir)

        # export indexed profile data
        coverage_file = coverage_dir.joinpath("sqlx.lcov")
        coverage_file.write_bytes(check_output([
            "cargo", "cov", "--", "export",
            "--format=lcov",
            "-Xdemangler=rustfilt",
            "--ignore-filename-regex", "/.cargo/registry",
            "--ignore-filename-regex", "/rustc/",
            "--instr-profile", f"{project_dir}/.coverage/sqlx.profdata",
            *map(lambda fn: f"--object={fn}", test_object_filenames),
        ], cwd=project_dir))

        # generate HTML coverage report
        check_output(["genhtml", "-o", coverage_dir, coverage_file], cwd=project_dir)


if __name__ == '__main__':
    main()
