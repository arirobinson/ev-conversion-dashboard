/**
 * @type {import('next').NextConfig}
 */
const nextConfig = {
    output: 'export',
    // Optional: Add a trailing slash to all paths `/about` -> `/about/`
    // trailingSlash: true,
    // Optional: Change the output directory `out` -> `dist`
    distDir: '_static',
    transpilePackages: ['@mui/x-charts'],
  }
   
  module.exports = nextConfig