export function normalizePathSeparators(path: string): string {
  return path.replace(/\\/g, '/')
}

export function basenameFromPath(path: string): string {
  if (!path) return ''

  const normalized = normalizePathSeparators(path).replace(/\/+$/, '')
  if (!normalized) return ''

  const lastSlashIndex = normalized.lastIndexOf('/')
  return lastSlashIndex >= 0 ? normalized.slice(lastSlashIndex + 1) : normalized
}
