#!/usr/bin/env python3
"""Test the source-built hook against checksum-pinned, real Mojang authlib API generations.
Downloads only official version metadata and selected libraries into a test cache.
No Minecraft launch, screenshots, account login or external skin requests.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import urllib.request

ROOT = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('agent_build', ROOT.parent / 'elyby-preview-compat/build.py')
builder = importlib.util.module_from_spec(spec)
spec.loader.exec_module(builder)


def fetch(url, checksum, target):
    if not target.is_file() or hashlib.sha1(target.read_bytes()).hexdigest() != checksum:
        data = urllib.request.urlopen(url, timeout=30).read()
        if hashlib.sha1(data).hexdigest() != checksum:
            raise ValueError(f'Checksum mismatch: {url}')
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)
    return target


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cache-dir', type=Path, default=ROOT.parents[1] / 'target/offline-skin-tests')
    parser.add_argument('--java', default='java', help='JDK 25+ runtime for the full matrix')
    parser.add_argument('--legacy-java', help='Optional Java 8 executable for legacy/older authlib coverage')
    parser.add_argument('--versions', nargs='+', help='Run only these Minecraft versions')
    args = parser.parse_args()
    versions = json.loads((ROOT / 'test-matrix.json').read_text())
    if args.versions:
        versions = [entry for entry in versions if entry['minecraft'] in args.versions]
        if len(versions) != len(set(args.versions)):
            parser.error('Version is not in test-matrix.json')
    with tempfile.TemporaryDirectory() as temporary:
        work = Path(temporary)
        agent = work / 'offline-skin.jar'
        subprocess.run([sys.executable, str(ROOT / "build.py"), "--output", str(agent)], check=True, capture_output=True)
        subprocess.run([builder.javac(), '--release', '8', '-Xlint:-options', '-d', str(work),
                        str(ROOT / 'tests/AuthlibTest.java')], check=True)
        png = work / 'skin.png'
        subprocess.run([args.java, '-Djava.awt.headless=true', '-cp', str(work), 'AuthlibTest', 'prepare', str(png)], check=True)
        for entry in versions:
            version = entry['minecraft']
            metadata = json.loads(fetch(entry['url'], entry['sha1'], args.cache_dir / f'{version}.json').read_text())
            libraries = []
            authlib = None
            for library in metadata.get('libraries', []):
                name = library['name']
                if not name.startswith(('com.mojang:authlib:', 'com.google.guava:', 'com.google.code.gson:',
                                        'org.apache.logging.log4j:', 'org.slf4j:', 'org.apache.commons:commons-lang3:', 'commons-io:')):
                    continue
                artifact = library.get('downloads', {}).get('artifact')
                if artifact:
                    libraries.append(fetch(artifact['url'], artifact['sha1'], args.cache_dir / 'libraries' / artifact['path']))
                if name.startswith('com.mojang:authlib:'):
                    authlib = name.split(':')[-1]
            legacy = authlib is None or authlib.startswith("1.3")
            if legacy:
                client = metadata['downloads']['client']
                jar = fetch(client['url'], client['sha1'], args.cache_dir / f'{version}-client.jar')
                subprocess.run([args.java, f'-javaagent:{agent}', '-Doneforall.localSkin.offlineAccount=false', '-cp', os.pathsep.join(map(str, [work, agent])),
                                'AuthlibTest', 'audit', str(jar)], check=True)
            runtimes = [args.java]
            if args.legacy_java and (authlib is None or authlib.startswith(('1.', '2.'))):
                runtimes.append(args.legacy_java)
            for java in runtimes:
                # All modes are separate JVMs so agent-disabled tests cannot inherit a server or transformed classes.
                for enabled, slim in [(True, True), (True, False), (False, False)]:
                    command = [java, '-Xverify:all', '-Djava.awt.headless=true', f'-javaagent:{agent}',
                               f'-Doneforall.localSkin.offlineAccount={str(enabled).lower()}',
                               f'-Doneforall.localSkin.path={png}', f'-Doneforall.localSkin.slim={str(slim).lower()}',
                               '-Doneforall.localSkin.uuid=12345678-1234-1234-1234-123456789abc',
                               '-Doneforall.localSkin.username=Player', '-cp',
                               os.pathsep.join(map(str, [work, agent, *libraries])),
                               'AuthlibTest', str(enabled).lower(), str(slim).lower(), str(png),
                               'legacy' if legacy else 'authlib']
                    result = subprocess.run(command, capture_output=True, text=True, timeout=30)
                    if result.returncode:
                        raise RuntimeError(f'{version} / {java} failed:\n{result.stdout}\n{result.stderr}')
                print(f'PASS Minecraft {version}, authlib {authlib or "legacy URLs"}, {java}: slim/classic/disabled', flush=True)


if __name__ == '__main__':
    main()
