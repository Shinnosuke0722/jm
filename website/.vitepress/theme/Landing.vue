<script setup lang="ts">
import { ref } from 'vue'
import { withBase } from 'vitepress'

const installCommand =
  'curl -fsSL https://raw.githubusercontent.com/Shinnosuke0722/jm/main/install.sh | sh'
const copied = ref(false)

async function copyInstallCommand() {
  if (typeof navigator === 'undefined') return

  await navigator.clipboard.writeText(installCommand)
  copied.value = true
  window.setTimeout(() => {
    copied.value = false
  }, 1800)
}
</script>

<template>
  <main class="jm-landing">
    <section class="jm-hero" aria-labelledby="jm-hero-title">
      <div class="jm-hero__glow" aria-hidden="true"></div>
      <div class="jm-shell">
        <div class="jm-hero__copy">
          <p class="jm-eyebrow"><span>jm</span> / Java Manager</p>
          <h1 id="jm-hero-title">Java Manager.<br />Every platform.</h1>
          <p class="jm-hero__lede">
            That is the idea behind jm: a native Rust CLI for installing, switching, and pinning JDK
            versions across Linux, macOS, and Windows.
          </p>
          <div class="jm-hero__actions">
            <a class="jm-button jm-button--primary" :href="withBase('/guide/getting-started.html')">
              Get started
              <span aria-hidden="true">→</span>
            </a>
            <a class="jm-button jm-button--quiet" href="https://github.com/Shinnosuke0722/jm">
              View on GitHub
            </a>
          </div>
          <ul class="jm-platforms" aria-label="Supported operating systems">
            <li>Linux</li>
            <li>macOS</li>
            <li>Windows</li>
            <li>x86-64 + ARM64*</li>
          </ul>
        </div>

        <div class="jm-resolver" aria-label="How jm resolves a project JDK">
          <div class="jm-resolver__topline">
            <span>resolution trace</span>
            <span class="jm-status"><i></i> ready</span>
          </div>

          <div class="jm-route">
            <article class="jm-route__node jm-route__node--source">
              <p>project input</p>
              <strong>.java-version</strong>
              <code>temurin-21</code>
            </article>

            <div class="jm-route__rail" aria-hidden="true">
              <span></span>
              <b>jm</b>
              <span></span>
            </div>

            <article class="jm-route__node jm-route__node--target">
              <p>active runtime</p>
              <strong>JDK 21</strong>
              <code>JAVA_HOME set</code>
            </article>
          </div>

          <div class="jm-terminal">
            <div class="jm-terminal__bar" aria-hidden="true">
              <span></span><span></span><span></span>
            </div>
            <code><em>$</em> jm current</code>
            <code><b>→</b> temurin-21.0.10+7</code>
          </div>
        </div>
      </div>
    </section>

    <section class="jm-install" aria-labelledby="jm-install-title">
      <div class="jm-shell jm-install__layout">
        <div>
          <p class="jm-section-label">Install</p>
          <h2 id="jm-install-title">Start with the binary.<br />Add the hook when you need it.</h2>
        </div>
        <div class="jm-command">
          <div class="jm-command__meta">
            <span>Linux / macOS</span>
            <button type="button" @click="copyInstallCommand">
              {{ copied ? 'Copied' : 'Copy' }}
            </button>
          </div>
          <code>{{ installCommand }}</code>
          <p class="sr-only" aria-live="polite">
            {{ copied ? 'Install command copied.' : '' }}
          </p>
          <a :href="withBase('/guide/getting-started.html')">Windows and source installation →</a>
        </div>
      </div>
    </section>

    <section class="jm-paths" aria-labelledby="jm-paths-title">
      <div class="jm-shell">
        <div class="jm-section-head">
          <div>
            <p class="jm-section-label">Choose a path</p>
            <h2 id="jm-paths-title">Documentation shaped around the work.</h2>
          </div>
          <p>
            Each guide starts with a developer task and stays explicit about platform behavior,
            precedence, and fallback rules.
          </p>
        </div>

        <div class="jm-doc-grid">
          <a class="jm-doc-link jm-doc-link--wide" :href="withBase('/guide/windows.html')">
            <span class="jm-doc-link__index">WIN</span>
            <div>
              <h3>JDK version manager for Windows</h3>
              <p>Install the x86-64 release, enable PowerShell, and troubleshoot PATH.</p>
            </div>
            <b aria-hidden="true">↗</b>
          </a>
          <a class="jm-doc-link" :href="withBase('/guide/project-switching.html')">
            <span class="jm-doc-link__index">PATH</span>
            <div>
              <h3>Switch Java versions per project</h3>
              <p>Resolve `.java-version`, `.sdkmanrc`, and the global default.</p>
            </div>
            <b aria-hidden="true">↗</b>
          </a>
          <a class="jm-doc-link" :href="withBase('/guide/sdkman-migration.html')">
            <span class="jm-doc-link__index">MOVE</span>
            <div>
              <h3>SDKMAN alternative on Windows</h3>
              <p>Carry the Java entry forward without claiming full SDKMAN compatibility.</p>
            </div>
            <b aria-hidden="true">↗</b>
          </a>
        </div>
      </div>
    </section>

    <section class="jm-distributions" aria-labelledby="jm-distributions-title">
      <div class="jm-shell jm-distributions__layout">
        <div>
          <p class="jm-section-label">OpenJDK catalog</p>
          <h2 id="jm-distributions-title">Ask for the build you need.</h2>
          <p>
            Search the provider catalog by distribution, Java version, operating system, and
            architecture.
          </p>
          <a :href="withBase('/guide/jdk-distributions.html')">
            Manage Temurin, Corretto, and GraalVM →
          </a>
        </div>
        <ul>
          <li><span>01</span> Temurin</li>
          <li><span>02</span> Corretto</li>
          <li><span>03</span> Zulu</li>
          <li><span>04</span> Liberica</li>
          <li><span>05</span> Microsoft OpenJDK</li>
          <li><span>06</span> GraalVM CE</li>
        </ul>
      </div>
    </section>

    <p class="jm-arm-note jm-shell">
      * Windows ARM64 does not currently receive a prebuilt release artifact. See the platform guide
      for source-build notes.
    </p>
  </main>
</template>
