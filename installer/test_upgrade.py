"""Exercise Windows upgrades from a released CLI using the local installer."""

import functools
import http.server
import os
import shutil
import subprocess
import tempfile
import threading
import urllib.request
import zipfile
from pathlib import Path

OLD_VERSION = "0.659.0"
NEW_VERSION = "0.660.0"
ARCHIVE = "qlty-x86_64-pc-windows-msvc.zip"
RELEASES_URL = "https://qlty-releases.s3.amazonaws.com/qlty"


def test_upgrade(root):
    fixtures = root / "fixtures"
    for version in (OLD_VERSION, NEW_VERSION):
        release = fixtures / f"v{version}"
        release.mkdir(parents=True)
        with urllib.request.urlopen(
            f"{RELEASES_URL}/v{version}/{ARCHIVE}", timeout=60
        ) as response, (release / ARCHIVE).open("wb") as archive:
            shutil.copyfileobj(response, archive)

    binary_dir = root / "bin"
    with zipfile.ZipFile(fixtures / f"v{OLD_VERSION}" / ARCHIVE) as archive:
        archive.extract("qlty.exe", binary_dir)

    # QLTY_INSTALL_URL serves both the installer and release archives.
    shutil.copyfile(Path(__file__).with_name("install.ps1"), fixtures / "index.html")
    handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=fixtures)
    with http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler) as server:
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            env = {
                **os.environ,
                "QLTY_INSTALL_URL": f"http://127.0.0.1:{server.server_port}",
                "QLTY_INSECURE_ALLOW_HTTP": "true",
                "QLTY_TELEMETRY": "off",
                "NO_COLOR": "1",
            }

            def run(*args):
                result = subprocess.run(
                    [str(binary_dir / "qlty.exe"), *args],
                    cwd=root,
                    env=env,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.STDOUT,
                    text=True,
                    timeout=120,
                )
                print(result.stdout, flush=True)
                return result

            def assert_version(expected):
                result = run("version", "--no-upgrade-check")
                assert result.returncode == 0, result.stdout
                assert result.stdout.split()[1] == expected, result.stdout

            assert_version(OLD_VERSION)

            failed = run("upgrade", "--version", "0.0.0", "--force")
            assert failed.returncode != 0, "Failed installer reported success"
            assert "Welcome to qlty" not in failed.stdout, failed.stdout
            assert_version(OLD_VERSION)

            upgraded = run("upgrade", "--version", NEW_VERSION, "--force")
            assert upgraded.returncode == 0, upgraded.stdout
            assert f"Welcome to qlty v{NEW_VERSION}!" in upgraded.stdout, upgraded.stdout
            assert_version(NEW_VERSION)
        finally:
            server.shutdown()
            thread.join()


if __name__ == "__main__":
    with tempfile.TemporaryDirectory(prefix="qlty-upgrade-") as temp_dir:
        test_upgrade(Path(temp_dir))
