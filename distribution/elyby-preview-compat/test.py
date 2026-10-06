#!/usr/bin/env python3
"""Console-only regression test; reads cached libraries/mods without modifying them."""
import argparse
import os
import pathlib
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--libraries", required=True, type=pathlib.Path)
parser.add_argument("--vanillahud", required=True, type=pathlib.Path)
parser.add_argument("--java", action="append", help="Java executable to test; repeat to test several runtimes")
args = parser.parse_args()
root = pathlib.Path(__file__).resolve().parent
agent = root.parents[1] / "packages/oneclient_core/assets/elyby-preview-compat.jar"
libraries = [agent, *[next(args.libraries.rglob(name)) for name in
    ["authlib-1.5.21.jar", "guava-17.0.jar", "gson-2.10.jar", "commons-lang3-3.3.2.jar"]]]
classpath = os.pathsep.join(map(str, libraries))
with tempfile.TemporaryDirectory() as directory:
    subprocess.run(["javac", "--release", "8", "-Xlint:-options", "-cp", classpath,
                    "-d", directory, *map(str, sorted((root / "tests").rglob("*.java")))], check=True)
    for java in args.java or ["java"]:
        subprocess.run([java, "-javaagent:" + str(agent), "-cp",
                        directory + os.pathsep + classpath, "CompatTest", str(args.vanillahud)], check=True)
