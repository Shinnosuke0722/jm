import { readFile, stat } from 'node:fs/promises'
import { resolve } from 'node:path'

const output = resolve('.vitepress/dist')
const siteUrl = 'https://shinnosuke0722.github.io/jm/'
const pages = [
  ['index.html', siteUrl],
  ['guide/getting-started.html', `${siteUrl}guide/getting-started.html`],
  ['guide/windows.html', `${siteUrl}guide/windows.html`],
  ['guide/project-switching.html', `${siteUrl}guide/project-switching.html`],
  ['guide/sdkman-migration.html', `${siteUrl}guide/sdkman-migration.html`],
]

function requireText(source, expected, file) {
  if (!source.includes(expected)) {
    throw new Error(`${file} is missing: ${expected}`)
  }
}

function forbidText(source, unexpected, file) {
  if (source.includes(unexpected)) {
    throw new Error(`${file} unexpectedly contains: ${unexpected}`)
  }
}

for (const [file, canonical] of pages) {
  const html = await readFile(resolve(output, file), 'utf8')
  requireText(html, '<meta name="description"', file)
  requireText(html, `<link rel="canonical" href="${canonical}">`, file)
  requireText(html, '<meta property="og:title"', file)
  requireText(html, '<meta property="og:description"', file)
  requireText(html, `<meta property="og:url" content="${canonical}">`, file)
  requireText(html, `${siteUrl}social-preview.jpg`, file)
}

const home = await readFile(resolve(output, 'index.html'), 'utf8')
requireText(home, '<h1 id="jm-hero-title">Java Manager.', 'index.html')
requireText(home, '<script type="application/ld+json">', 'index.html')
requireText(home, '"@type":"SoftwareApplication"', 'index.html')
requireText(home, 'src="/jm/jm-mark.svg"', 'index.html')
for (const [, canonical] of pages.slice(1)) {
  requireText(home, `href="${new URL(canonical).pathname}"`, 'index.html')
}

const projectSwitching = await readFile(resolve(output, 'guide/project-switching.html'), 'utf8')
requireText(projectSwitching, 'id="enable-the-shell-hook"', 'guide/project-switching.html')

const sdkmanMigration = await readFile(resolve(output, 'guide/sdkman-migration.html'), 'utf8')
requireText(
  sdkmanMigration,
  'href="./project-switching.html#enable-the-shell-hook"',
  'guide/sdkman-migration.html',
)

for (const file of [
  'guide/windows.html',
  'guide/project-switching.html',
  'guide/sdkman-migration.html',
]) {
  const html = await readFile(resolve(output, file), 'utf8')
  forbidText(html, 'class="last-updated"', file)
}

const sitemap = await readFile(resolve(output, 'sitemap.xml'), 'utf8')
for (const [, canonical] of pages) {
  requireText(sitemap, `<loc>${canonical}</loc>`, 'sitemap.xml')
}

const socialPreview = await stat(resolve(output, 'social-preview.jpg'))
if (socialPreview.size >= 1024 * 1024) {
  throw new Error(`social-preview.jpg is ${socialPreview.size} bytes; GitHub requires under 1 MiB`)
}

console.log(`Verified ${pages.length} pages, sitemap, and Social Preview.`)
