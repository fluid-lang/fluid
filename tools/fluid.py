"""
fluid.py is a small python script that helps to build The Fluid Programming Language easily.

usage: fluid.py [-h] {build, test, check} ...

Note: While contributing to this script the applied changes should be compatible with both python 2 and python 3
    as most of the operating systems come with python 2 by default.
"""
from __future__ import print_function

import argparse
import os
import sys

stderr = {
    "errors_vec": [],
}

verbose = False


def error(err):
    global error

    print("\033[1;31merror:\033[0m " + err, file=sys.stderr)

    stderr["errors_vec"].append(err)


class FluidBuild:
    def __init__(self, args):
        build_type = args["[RELEASE | DEBUG]"].lower()
        run = args["run"]

        if not build_type in ["release", "debug"]:
            error(
                "Expected the build type to be `release` or `debug` but found `" + build_type + "`")

        self.flags = ["--release" if build_type == "release" else ""]
        self.command = "run" if run else "build"

    def run(self):
        os.system("cargo " + self.command + " " + " ".join(self.flags))


class FluidTest:
    def __init__(self, args):
        test_type = args["[RELEASE | DEBUG]"].lower()

        if not test_type in ["release", "debug"]:
            error(
                "Expected the test type to be `release` or `debug` but found `" + test_type + "`")

        self.flags = ["--release" if test_type == "release" else ""]

    def run(self):
        os.system("cargo test " + " ".join(self.flags))


class FluidPreCheck:
    def __init__(self, args): ()

    def run(self):
        for package in os.listdir("./packages/"):
            if os.system("cd ./packages/" + package + " && cargo test") != 0:
                error("Failed running `cargo test` for " + package)

        if os.system("cargo fmt -- --check") != 0:
            error("Failed running `cargo fmt -- --check`")


def main():
    global stderr

    parser = argparse.ArgumentParser(
        description="A small python script that helps to build Fluid Lang easily")

    subparser = parser.add_subparsers(dest="command")

    build = subparser.add_parser(name="build", description="Build Fluid Lang")
    build.add_argument("[RELEASE | DEBUG]")
    build.add_argument(
        "--run", "-r", action="store_true")

    test = subparser.add_parser(name="test", description="Test Fluid Lang")
    test.add_argument("[RELEASE | DEBUG]")

    subparser.add_parser(
        name="check", description="Run tests, run clippy, check formatting, and then run a test release build.")

    args = vars(parser.parse_args())

    commands = {
        "build": FluidBuild,
        "test": FluidTest,
        "check": FluidPreCheck
    }

    if args["command"] != None:
        commands[args["command"]](args).run()

    if len(stderr["errors_vec"]) > 0:
        print("\n\033[1;31merror:\033[0m Aborting due to previous " + ("error" if len(
            stderr["errors_vec"]) == 1 else str(len(stderr["errors_vec"])) + " errors"))


if __name__ == "__main__":
    main()
