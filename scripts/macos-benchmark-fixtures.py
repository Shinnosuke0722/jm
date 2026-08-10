#!/usr/bin/env python3
"""Create isolated, equivalent local-JDK fixtures for the macOS benchmark."""

import json
import os
import platform
import shutil
from pathlib import Path


root = Path(os.environ["BENCH_ROOT"]).resolve()
java_arch = "aarch64" if platform.machine() == "arm64" else "x86_64"
sdkman_template = Path(os.environ["SDKMAN_TEMPLATE"]).resolve()


def write_jdk(path: Path, version: str, release: bool = False) -> None:
    java = path / "bin" / "java"
    java.parent.mkdir(parents=True, exist_ok=True)
    java.write_text("#!/bin/sh\nexit 0\n", encoding="ascii")
    java.chmod(0o755)
    if release:
        (path / "release").write_text(
            f'JAVA_VERSION="{version}"\nJAVA_VENDOR="Eclipse Adoptium"\n'
            f'OS_ARCH="{java_arch}"\n',
            encoding="ascii",
        )


for scale in (0, 10, 100, 1000):
    fixture = root / "fixtures" / f"n{scale}"
    jm_home = fixture / "jm"
    jabba_home = fixture / "jabba"
    javm_home = fixture / "javm"
    mise_data = fixture / "mise-data"
    sdkman_dir = fixture / "sdkman"
    shutil.copytree(sdkman_template, sdkman_dir, dirs_exist_ok=True)
    (sdkman_dir / "candidates" / "java").mkdir(parents=True, exist_ok=True)
    for path in (jm_home, jabba_home / "jdk", javm_home / "jdk", mise_data / "installs" / "java", fixture / "cwd"):
        path.mkdir(parents=True, exist_ok=True)

    installations = []
    for index in range(1, scale + 1):
        version = f"21.0.{index}"
        identifier = f"temurin-{version}"
        jm_path = jm_home / "jdks" / identifier
        write_jdk(jm_path, version)
        installations.append({
            "id": identifier,
            "distribution": "temurin",
            "java_version": {"major": 21, "minor": 0, "patch": index, "build": None},
            "full_version": version,
            "major_version": 21,
            "path": str(jm_path),
            "installed_at": "2026-08-11T00:00:00Z",
            "is_lts": True,
        })
        write_jdk(jabba_home / "jdk" / f"temurin@{version}", version)
        write_jdk(javm_home / "jdk" / f"temurin@{version}", version, True)
        write_jdk(mise_data / "installs" / "java" / version, version)
        write_jdk(sdkman_dir / "candidates" / "java" / f"{version}-tem", version)

    (jm_home / "registry.json").write_text(json.dumps({"format_version": 1, "installations": installations}), encoding="utf-8")
    autodiscover = javm_home / "autodiscover"
    autodiscover.mkdir(exist_ok=True)
    (autodiscover / "config.json").write_text(json.dumps({"enabled": True, "sources": {"system": False, "jabba": False, "gradle": False, "intellij": False, "javm": True}, "cache_ttl": 86400000000000}), encoding="utf-8")
    if scale:
        (sdkman_dir / "candidates" / "java" / "current").symlink_to(f"{version}-tem")

current = root / "fixtures" / "n10" / "jm" / "current"
current.symlink_to(root / "fixtures" / "n10" / "jm" / "jdks" / "temurin-21.0.1")

for depth in (0, 5, 20, 50, 100):
    project = root / "fixtures" / "projects" / f"d{depth}"
    project.mkdir(parents=True, exist_ok=True)
    (project / ".java-version").write_text("21.0.1\n", encoding="ascii")
    (project / ".jabbarc").write_text("temurin@21.0.1\n", encoding="ascii")
    (project / "mise.toml").write_text('[tools]\njava = "21.0.1"\n', encoding="ascii")
    (project / ".sdkmanrc").write_text("java=21.0.1-tem\n", encoding="ascii")
    leaf = project
    for _ in range(depth):
        leaf /= "d"
    leaf.mkdir(parents=True, exist_ok=True)
    (project / "leaf.txt").write_text(str(leaf), encoding="utf-8")
