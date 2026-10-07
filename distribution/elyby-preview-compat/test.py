#!/usr/bin/env python3
"""Console-only regression test; reads cached libraries/mods without modifying them."""
import argparse
import os
import pathlib
import subprocess
import tempfile
from build import javac

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--libraries", required=True, type=pathlib.Path)
parser.add_argument("--vanillahud", required=True, type=pathlib.Path)
parser.add_argument("--agent", type=pathlib.Path, default=pathlib.Path(__file__).resolve().parents[2] / "target/elyby-preview-compat/elyby-preview-compat.jar")
parser.add_argument("--java", action="append", help="Java executable to test; repeat to test several runtimes")
args = parser.parse_args()
root = pathlib.Path(__file__).resolve().parent
agent = args.agent
if not agent.is_file():
    parser.error("Build the agent first with build.py, or pass --agent pointing to Cargo\'s generated JAR")
libraries = [agent, *[next(args.libraries.rglob(name)) for name in
    ["authlib-1.5.21.jar", "guava-17.0.jar", "gson-2.10.jar", "commons-lang3-3.3.2.jar"]]]
classpath = os.pathsep.join(map(str, libraries))
with tempfile.TemporaryDirectory() as directory:
    subprocess.run([javac(), "--release", "8", "-Xlint:-options", "-cp", classpath,
                    "-d", directory, *map(str, sorted((root / "tests").rglob("*.java")))], check=True)
    for java in args.java or ["java"]:
        subprocess.run([java, "-javaagent:" + str(agent), "-cp",
                        directory + os.pathsep + classpath, "CompatTest", str(args.vanillahud)], check=True)
