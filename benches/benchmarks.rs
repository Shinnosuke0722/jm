use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_version_parsing(c: &mut Criterion) {
    use jm_core::java_version::JavaVersion;

    let mut group = c.benchmark_group("version_parsing");

    group.bench_function("modern_full", |b| {
        b.iter(|| JavaVersion::parse(black_box("21.0.2+13")))
    });

    group.bench_function("major_only", |b| {
        b.iter(|| JavaVersion::parse(black_box("21")))
    });

    group.bench_function("legacy_format", |b| {
        b.iter(|| JavaVersion::parse(black_box("1.8.0_362")))
    });

    group.finish();
}

fn bench_version_spec_parsing(c: &mut Criterion) {
    use jm_core::java_version::VersionSpec;

    let mut group = c.benchmark_group("version_spec_parsing");

    group.bench_function("with_distribution", |b| {
        b.iter(|| VersionSpec::parse(black_box("temurin-21.0.2+13")))
    });

    group.bench_function("bare_version", |b| {
        b.iter(|| VersionSpec::parse(black_box("21")))
    });

    group.bench_function("compound_distribution", |b| {
        b.iter(|| VersionSpec::parse(black_box("graalvm-ce-22")))
    });

    group.finish();
}

fn bench_distribution_parsing(c: &mut Criterion) {
    use jm_core::distribution::Distribution;

    let mut group = c.benchmark_group("distribution_parsing");

    group.bench_function("known", |b| {
        b.iter(|| Distribution::parse(black_box("temurin")))
    });

    group.bench_function("alias", |b| {
        b.iter(|| Distribution::parse(black_box("adoptium")))
    });

    group.bench_function("unknown", |b| {
        b.iter(|| Distribution::parse(black_box("custom-dist")))
    });

    group.finish();
}

fn bench_registry_operations(c: &mut Criterion) {
    use jm_core::distribution::Distribution;
    use jm_core::java_version::JavaVersion;
    use jm_core::registry::{Installation, Registry};
    use std::path::PathBuf;

    let mut group = c.benchmark_group("registry");

    // Build a registry with 20 entries (realistic)
    let mut reg = Registry::empty();
    for i in 0..20 {
        let major = 11 + (i % 4) * 4; // 11, 15, 19, 23
        let dists = ["temurin", "corretto", "zulu", "liberica", "microsoft"];
        let dist_name = dists[i % dists.len()];
        let id = format!("{}-{}.0.{}", dist_name, major, i);
        reg.add(Installation {
            id,
            distribution: Distribution::parse(dist_name),
            java_version: JavaVersion {
                major: major as u32,
                minor: Some(0),
                patch: Some(i as u32),
                build: None,
            },
            full_version: format!("{}.0.{}", major, i),
            major_version: major as u32,
            path: PathBuf::from(format!("/tmp/jdks/jdk-{}", i)),
            installed_at: chrono::Utc::now(),
            is_lts: major == 11 || major == 21,
        });
    }

    group.bench_function("find_by_id", |b| {
        b.iter(|| reg.find_by_id(black_box("temurin-21.0.10")))
    });

    group.bench_function("find_matching_major", |b| {
        let spec = JavaVersion::new(21);
        b.iter(|| reg.find_matching(None, black_box(&spec)))
    });

    group.bench_function("sorted", |b| b.iter(|| reg.sorted()));

    // Serialization benchmark
    group.bench_function("serialize_json", |b| {
        b.iter(|| serde_json::to_string(black_box(&reg)).unwrap())
    });

    let json = serde_json::to_string(&reg).unwrap();
    group.bench_function("deserialize_json", |b| {
        b.iter(|| serde_json::from_str::<Registry>(black_box(&json)).unwrap())
    });

    // Save to temp file
    group.bench_function("save_to_file", |b| {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("registry.json");
        b.iter(|| reg.save_to(black_box(&path)).unwrap())
    });

    group.finish();
}

fn bench_config_operations(c: &mut Criterion) {
    use jm_core::config::Config;

    let mut group = c.benchmark_group("config");

    group.bench_function("default", |b| b.iter(Config::default));

    group.bench_function("serialize_toml", |b| {
        let config = Config::default();
        b.iter(|| toml::to_string_pretty(black_box(&config)).unwrap())
    });

    let toml_str = toml::to_string_pretty(&Config::default()).unwrap();
    group.bench_function("deserialize_toml", |b| {
        b.iter(|| toml::from_str::<Config>(black_box(&toml_str)).unwrap())
    });

    group.bench_function("load_from_file", |b| {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        Config::default().save_to(&path).unwrap();
        b.iter(|| Config::load_from(black_box(&path)).unwrap())
    });

    group.finish();
}

fn bench_shell_init(c: &mut Criterion) {
    use jm_shell::hook::Shell;

    let mut group = c.benchmark_group("shell_init");

    group.bench_function("bash", |b| {
        b.iter(|| Shell::Bash.init_script(black_box("/home/user/.jm")))
    });

    group.bench_function("zsh", |b| {
        b.iter(|| Shell::Zsh.init_script(black_box("/home/user/.jm")))
    });

    group.bench_function("fish", |b| {
        b.iter(|| Shell::Fish.init_script(black_box("/home/user/.jm")))
    });

    group.bench_function("powershell", |b| {
        b.iter(|| Shell::PowerShell.init_script(black_box("/home/user/.jm")))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_version_parsing,
    bench_version_spec_parsing,
    bench_distribution_parsing,
    bench_registry_operations,
    bench_config_operations,
    bench_shell_init,
);
criterion_main!(benches);
