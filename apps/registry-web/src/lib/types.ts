export interface Package {
  name: string
  version: string
  description: string
  author: string
  license: string
  tags: Array<string>
  tsx_min: string
  created_at: string
  updated_at: string
  download_count: number
  lang?: string
  runtime?: string
  provides?: Array<string>
  integrates_with?: Array<string>
}

export interface PackageVersion {
  version: string
  published_at: string
  download_count: number
}

export interface SearchResult {
  packages: Array<Package>
  total: number
  page: number
  per_page: number
}

export interface RegistryStats {
  total_packages: number
  total_downloads: number
  total_versions: number
  packages_this_week: number
}
