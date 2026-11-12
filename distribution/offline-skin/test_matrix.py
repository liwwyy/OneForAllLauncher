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

# Test-only artifacts from Fabric's own Maven, pinned independently of game data.
FABRIC = [
    ("fabric-loader-0.20.1.jar", "https://maven.fabricmc.net/net/fabricmc/fabric-loader/0.20.1/fabric-loader-0.20.1.jar",
     "d0c5a082e07410aaff9de7ab196aa6b4236002870f9ba3ad5c1260fcce5110ad"),
    ("sponge-mixin.jar", "https://maven.fabricmc.net/net/fabricmc/sponge-mixin/0.17.5+mixin.0.8.7/sponge-mixin-0.17.5+mixin.0.8.7.jar",
     "03f0065023bfc3045d71bfb99872451d9ad5a0800a5a0dac34ab8c7d5c134f5d"),
]

SECURE_SKINS_URL = "https://cdn.modrinth.com/data/4C4XkPCE/versions/cFBMxP98/secureskins-1.0.1%2B26.1.1-fabric.jar"
SECURE_SKINS_SHA1 = "7d791989170b504270940aca1aaa690689cd41c0"
ONECONFIG_HEAD_URL = "https://cdn.modrinth.com/data/AibBIVmj/versions/uXzmmgO2/OneConfig-26.3-fabric-1.7.0.jar"
ONECONFIG_HEAD_SHA1 = "b96e5deb7ae1ee6fe5d3da1dab9f44e967081832"
KOTLIN_URL = "https://repo.maven.apache.org/maven2/org/jetbrains/kotlin/kotlin-stdlib/2.4.21/kotlin-stdlib-2.4.21.jar"
KOTLIN_SHA1 = "c343e135bd1e7dd94b7ed76b4159068d888d09db"


def fabric_libraries(cache):
    result = []
    for name, url, digest in FABRIC:
        path = cache / 'fabric' / name
        if not path.is_file() or hashlib.sha256(path.read_bytes()).hexdigest() != digest:
            data = urllib.request.urlopen(url, timeout=30).read()
            if hashlib.sha256(data).hexdigest() != digest:
                raise ValueError(f'Checksum mismatch: {url}')
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        result.append(path)
    result.append(builder.verified_asm(builder.DEFAULT_CACHE))
    return result


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
    parser.add_argument('--fabric', action='store_true', help='Also test real Fabric 0.20.1 classloaders using the primary Java runtime (17+)')
    parser.add_argument('--secure-skins', action='store_true', help='Test the real Secure Skins 1.0.1 HTTPS rewrite on 26.3 (Java 25+)')
    parser.add_argument('--oneconfig-head', action='store_true', help='Test the shipped OneConfig PlayerHead reader used by PolyPlus on 26.3 (Java 25+)')
    args = parser.parse_args()
    versions = json.loads((ROOT / 'test-matrix.json').read_text())
    if args.versions:
        versions = [entry for entry in versions if entry['minecraft'] in args.versions]
        if len(versions) != len(set(args.versions)):
            parser.error('Version is not in test-matrix.json')
    if args.secure_skins and not any(entry['minecraft'] == '26.3' for entry in versions):
        parser.error('--secure-skins requires 26.3 in the selected versions')
    if args.oneconfig_head and not any(entry['minecraft'] == '26.3' for entry in versions):
        parser.error('--oneconfig-head requires 26.3 in the selected versions')
    with tempfile.TemporaryDirectory() as temporary:
        work = Path(temporary)
        agent = work / 'offline-skin.jar'
        subprocess.run([sys.executable, str(ROOT / "build.py"), "--output", str(agent)], check=True, capture_output=True)
        asm = builder.verified_asm(builder.DEFAULT_CACHE)
        subprocess.run([builder.javac(), '--release', '8', '-Xlint:-options', '-cp', str(asm), '-d', str(work),
                        str(ROOT / 'tests/AuthlibTest.java'), str(ROOT / 'tests/FabricLoaderTest.java'),
                        str(ROOT / 'tests/SecureSkinsTest.java'), str(ROOT / 'tests/OneConfigHeadTest.java')], check=True)
        fabric = fabric_libraries(args.cache_dir) if args.fabric else []
        secure_skins = fetch(SECURE_SKINS_URL, SECURE_SKINS_SHA1, args.cache_dir / 'mods/secureskins.jar') if args.secure_skins else None
        oneconfig = fetch(ONECONFIG_HEAD_URL, ONECONFIG_HEAD_SHA1, args.cache_dir / 'mods/oneconfig-head.jar') if args.oneconfig_head else None
        kotlin = fetch(KOTLIN_URL, KOTLIN_SHA1, args.cache_dir / 'libraries/kotlin-stdlib-2.4.21.jar') if args.oneconfig_head else None
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
                    if fabric and not legacy and java == args.java:
                        fabric_command = command[:command.index('-cp')]
                        fabric_command.extend(['-cp', os.pathsep.join(map(str, [work, agent, *libraries, *fabric])),
                            'FabricLoaderTest', os.pathsep.join(map(str, libraries)),
                            str(enabled).lower(), str(slim).lower(), str(png)])
                        result = subprocess.run(fabric_command, capture_output=True, text=True, timeout=30)
                        if result.returncode:
                            raise RuntimeError(f'{version} / Fabric failed:\n{result.stdout}\n{result.stderr}')
                    if secure_skins and version == '26.3' and enabled and java == args.java:
                        secure_command = command[:command.index('-cp')]
                        secure_command.extend(['-cp', os.pathsep.join(map(str, [work, agent, *libraries, asm])),
                            'SecureSkinsTest', str(png), str(secure_skins), str(slim).lower()])
                        result = subprocess.run(secure_command, capture_output=True, text=True, timeout=30)
                        if result.returncode:
                            raise RuntimeError(f'{version} / Secure Skins failed:\n{result.stdout}\n{result.stderr}')
                    if oneconfig and version == '26.3' and enabled and java == args.java:
                        head_command = command[:command.index('-cp')]
                        head_command.extend(['-cp', os.pathsep.join(map(str, [work, agent, *libraries, asm, kotlin])),
                            'OneConfigHeadTest', str(png), str(oneconfig), str(slim).lower()])
                        result = subprocess.run(head_command, capture_output=True, text=True, timeout=30)
                        if result.returncode:
                            raise RuntimeError(f'{version} / PlayerHead failed:\n{result.stdout}\n{result.stderr}')
                loaders = ', real Fabric Knot/compatibility loaders' if fabric and not legacy and java == args.java else ''
                print(f'PASS Minecraft {version}, authlib {authlib or "legacy URLs"}, {java}: slim/classic/disabled{loaders}', flush=True)
                if secure_skins and version == '26.3' and java == args.java:
                    print('PASS real Secure Skins 1.0.1: private PNG transport restored, remote HTTPS unchanged', flush=True)
                if oneconfig and version == '26.3' and java == args.java:
                    print('PASS shipped OneConfig 1.7.0 PlayerHead reader used by PolyPlus: private profile and skin PNG', flush=True)


if __name__ == '__main__':
    main()
