#!/usr/bin/env python3
import argparse
import subprocess
from os import environ
from pathlib import Path
from shutil import rmtree
from subprocess import check_call, check_output, PIPE

parser = argparse.ArgumentParser()
parser.add_argument("-v", "--verbose", action="store_true")
parser.add_argument("-t", "--target")
parser.add_argument("-l", "--list-targets", action="store_true")
parser.add_argument("--coverage", action="store_true")

argv, unknown = parser.parse_known_args()

# get an absolute path to the project directory for SQLx
project_dir = Path(__file__).parent.parent
coverage_dir = project_dir.joinpath(".coverage")
raw_coverage_dir = project_dir.joinpath("raw")

# global test filenames
# we capture these all so we can collect coverage later
test_object_filenames = []


def run(cmd, *, cwd, env=None, comment=None):
    if comment is not None:
        print(f"\x1b[2m # {comment}\x1b[0m")

    print(f"\x1b[93m $ {' '.join(cmd)}\x1b[0m")

    subprocess.run(cmd, env=env, cwd=project_dir, check=True,
                   stdout=None if argv.verbose else PIPE)


def run_check(project_name: str):
    project = f"sqlx-{project_name}"
    tag = f"check:{project_name}"

    if argv.target is not None and argv.target not in tag:
        return

    if argv.list_targets:
        print(tag)
        return

    run([
        "cargo", "check",
        "--manifest-path", f"{project}/Cargo.toml",
    ], cwd=project_dir, comment=f"check {project}")

    run([
        "cargo", "check",
        "--manifest-path", f"{project}/Cargo.toml",
        "--features", "blocking",
    ], cwd=project_dir, comment=f"check {project} +blocking")

    run([
        "cargo", "check",
        "--manifest-path", f"{project}/Cargo.toml",
        "--features", "async",
    ], cwd=project_dir, comment=f"check {project} +async")

    run([
        "cargo", "check",
        "--manifest-path", f"{project}/Cargo.toml",
        "--features", "blocking,async",
    ], cwd=project_dir, comment=f"check {project} +async,+blocking")


def run_clippy(project_name: str):
    project = f"sqlx-{project_name}"
    tag = f"clippy:{project_name}"

    if argv.target is not None and argv.target not in tag:
        return

    if argv.list_targets:
        print(tag)
        return

    # update timestamp so clippy will run
    Path(f"{project}/src/lib.rs").touch()

    run([
        "cargo", "clippy",
        "--manifest-path", f"{project}/Cargo.toml",
    ], cwd=project_dir, comment=f"clippy {project}")

    # update timestamp so clippy will run
    Path(f"{project}/src/lib.rs").touch()

    run([
        "cargo", "clippy",
        "--manifest-path", f"{project}/Cargo.toml",
        "--features", "blocking",
    ], cwd=project_dir, comment=f"clippy {project} +blocking")

    # update timestamp so clippy will run
    Path(f"{project}/src/lib.rs").touch()

    run([
        "cargo", "clippy",
        "--manifest-path", f"{project}/Cargo.toml",
        "--features", "async",
    ], cwd=project_dir, comment=f"clippy {project} +async")

    # update timestamp so clippy will run
    Path(f"{project}/src/lib.rs").touch()

    run([
        "cargo", "clippy",
        "--manifest-path", f"{project}/Cargo.toml",
        "--features", "blocking,async",
    ], cwd=project_dir, comment=f"clippy {project} +async,+blocking")


def run_unit_test(project_name: str):
    project = f"sqlx-{project_name}"
    tag = f"unit:{project_name}"

    if argv.target is not None and argv.target not in tag:
        return

    if argv.list_targets:
        print(tag)
        return

    env = environ.copy()
    env["LLVM_PROFILE_FILE"] = f"{project_dir}/.coverage/raw/{project}_%m.profraw"

    if argv.coverage:
        env["RUSTFLAGS"] = "-Zinstrument-coverage"

    # run the tests
    run([
        "cargo", "+nightly", "test",
        "--manifest-path", f"{project}/Cargo.toml",
        "--features", "async",
    ], env=env, cwd=project_dir, comment=f"unit test {project}")

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
    raw_coverage_dir = Path(f"{project_dir}/.coverage/raw/")
    rmtree(raw_coverage_dir)
    raw_coverage_dir.mkdir()

    # run checks
    run_check("core")
    run_check("mysql")

    # run checks
    run_clippy("core")
    run_clippy("mysql")

    # run unit tests, collect test binary filenames
    run_unit_test("core")
    run_unit_test("mysql")

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
