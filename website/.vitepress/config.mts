import { readFileSync } from 'node:fs'
import { defineConfig } from 'vitepress'

const base = '/jm/'
const siteUrl = 'https://shinnosuke0722.github.io/jm/'
const repositoryUrl = 'https://github.com/Shinnosuke0722/jm'
const socialImageUrl = `${siteUrl}social-preview.jpg`
const defaultDescription =
  'jm stands for Java Manager: a native Rust CLI for installing, switching, and pinning JDK versions across Linux, macOS, and Windows.'
const cargoManifest = readFileSync(new URL('../../Cargo.toml', import.meta.url), 'utf8')
const softwareVersion = cargoManifest.match(
  /\[workspace\.package\][\s\S]*?version\s*=\s*"([^"]+)"/,
)?.[1]

function canonicalUrl(relativePath: string) {
  const path = relativePath.replace(/index\.md$/, '').replace(/\.md$/, '.html')
  return new URL(path, siteUrl).href
}

export default defineConfig({
  lang: 'en-US',
  title: 'jm',
  titleTemplate: ':title — jm docs',
  description: defaultDescription,
  base,
  appearance: false,
  cleanUrls: false,
  lastUpdated: true,
  sitemap: {
    hostname: siteUrl,
  },
  head: [
    ['meta', { name: 'theme-color', content: '#111c2d' }],
    ['link', { rel: 'icon', type: 'image/svg+xml', href: `${base}jm-mark.svg` }],
    ['link', { rel: 'manifest', href: `${base}site.webmanifest` }],
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:site_name', content: 'jm documentation' }],
    ['meta', { property: 'og:image', content: socialImageUrl }],
    ['meta', { property: 'og:image:width', content: '1280' }],
    ['meta', { property: 'og:image:height', content: '640' }],
    [
      'meta',
      {
        property: 'og:image:alt',
        content: 'jm resolves a project Java version to an active JDK',
      },
    ],
    ['meta', { name: 'twitter:card', content: 'summary_large_image' }],
    ['meta', { name: 'twitter:image', content: socialImageUrl }],
  ],
  transformPageData(pageData) {
    if (pageData.relativePath === '404.md') return

    const description = pageData.description || defaultDescription
    const title =
      pageData.relativePath === 'index.md'
        ? 'jm — The cross-platform Java Manager'
        : `${pageData.title} — jm docs`
    const canonical = canonicalUrl(pageData.relativePath)

    pageData.frontmatter.head ??= []
    pageData.frontmatter.head.push(
      ['link', { rel: 'canonical', href: canonical }],
      ['meta', { property: 'og:title', content: title }],
      ['meta', { property: 'og:description', content: description }],
      ['meta', { property: 'og:url', content: canonical }],
      ['meta', { name: 'twitter:title', content: title }],
      ['meta', { name: 'twitter:description', content: description }],
    )

    if (pageData.relativePath === 'index.md') {
      pageData.frontmatter.head.push([
        'script',
        { type: 'application/ld+json' },
        JSON.stringify({
          '@context': 'https://schema.org',
          '@type': 'SoftwareApplication',
          name: 'jm',
          description: defaultDescription,
          applicationCategory: 'DeveloperApplication',
          operatingSystem: ['Linux', 'macOS', 'Windows'],
          ...(softwareVersion ? { softwareVersion } : {}),
          codeRepository: repositoryUrl,
          url: siteUrl,
          license: [
            'https://spdx.org/licenses/MIT.html',
            'https://spdx.org/licenses/Apache-2.0.html',
          ],
          offers: {
            '@type': 'Offer',
            price: '0',
            priceCurrency: 'USD',
          },
        }),
      ])
    }
  },
  themeConfig: {
    logo: '/jm-mark.svg',
    siteTitle: 'jm',
    nav: [
      { text: 'Get started', link: '/guide/getting-started.html' },
      { text: 'Windows', link: '/guide/windows.html' },
      { text: 'Project switching', link: '/guide/project-switching.html' },
      { text: 'SDKMAN migration', link: '/guide/sdkman-migration.html' },
      { text: 'GitHub', link: repositoryUrl },
    ],
    sidebar: {
      '/guide/': [
        {
          text: 'Start here',
          items: [{ text: 'Getting started', link: '/guide/getting-started.html' }],
        },
        {
          text: 'Workflows',
          items: [
            { text: 'Windows and PowerShell', link: '/guide/windows.html' },
            {
              text: 'Project JDK switching',
              link: '/guide/project-switching.html',
            },
            {
              text: 'Migrate from SDKMAN',
              link: '/guide/sdkman-migration.html',
            },
          ],
        },
      ],
    },
    search: {
      provider: 'local',
    },
    outline: {
      level: [2, 3],
      label: 'On this page',
    },
    lastUpdated: {
      text: 'Updated',
      formatOptions: {
        dateStyle: 'medium',
      },
    },
    socialLinks: [{ icon: 'github', link: repositoryUrl }],
    footer: {
      message: 'Native JDK management for Linux, macOS, and Windows.',
      copyright: 'Released under the MIT or Apache-2.0 license.',
    },
  },
})
