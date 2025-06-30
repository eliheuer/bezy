# GitHub Pages Deployment Guide

This guide explains how to deploy Bezy Font Editor to GitHub Pages using your custom domain (bezy.org) with automated GitHub Actions.

## Overview

The deployment system consists of:
- **GitHub Actions Workflow**: Automatically builds and deploys on push to main
- **Build Script**: Prepares WASM files and static assets for hosting
- **Custom Domain**: Serves the app from bezy.org
- **Static Hosting**: GitHub Pages serves the compiled WASM application

## Prerequisites

Before you begin, ensure you have:

1. **GitHub Repository**: Your Bezy repository pushed to GitHub
2. **Domain Name**: bezy.org (which you already have)
3. **DNS Configuration**: Access to configure DNS records for your domain
4. **Repository Permissions**: Admin access to enable GitHub Pages

## Step-by-Step Setup

### 1. Enable GitHub Pages

1. Go to your GitHub repository settings
2. Navigate to **Pages** in the left sidebar
3. Under **Source**, select "GitHub Actions"
4. This enables GitHub Actions to deploy to Pages

### 2. Configure DNS for Custom Domain

You need to point your domain to GitHub Pages. Add these DNS records:

#### Option A: Using CNAME (Recommended for subdomains)
```
Type: CNAME
Name: www
Value: yourusername.github.io
TTL: 3600
```

#### Option B: Using A Records (For apex domain)
```
Type: A
Name: @
Value: 185.199.108.153
TTL: 3600

Type: A  
Name: @
Value: 185.199.109.153
TTL: 3600

Type: A
Name: @
Value: 185.199.110.153
TTL: 3600

Type: A
Name: @
Value: 185.199.111.153
TTL: 3600
```

### 3. Push Your Code

The deployment is automated via GitHub Actions. Simply push to the `main` branch:

```bash
git add .
git commit -m "Setup GitHub Pages deployment"
git push origin main
```

### 4. Monitor Deployment

1. Go to the **Actions** tab in your GitHub repository
2. You'll see the "Deploy to GitHub Pages" workflow running
3. The build process typically takes 2-5 minutes
4. Once complete, your site will be live at bezy.org

## Files Created

The deployment setup includes these files:

### `.github/workflows/deploy.yml`
- GitHub Actions workflow
- Builds WASM on Ubuntu
- Caches dependencies for faster builds
- Deploys to GitHub Pages automatically

### `build-github-pages.sh`
- Builds the WASM version for production
- Creates optimized static files
- Generates proper HTML with loading screen
- Includes JavaScript bindings
- Sets up custom domain (CNAME)

### Key Features of the Build

- **Professional Loading Screen**: Shows while WASM loads
- **Error Handling**: Displays helpful messages if loading fails
- **Responsive Design**: Works on desktop and mobile
- **Asset Management**: Automatically includes fonts and other assets
- **SEO Friendly**: Proper meta tags and description

## Local Testing

Test your GitHub Pages build locally before deploying:

```bash
# Build for GitHub Pages
./build-github-pages.sh

# Serve locally (requires Python 3)
cd dist
python -m http.server 8080

# Open in browser
open http://localhost:8080
```

## Troubleshooting

### Build Fails
- Check the Actions tab for error logs
- Ensure all Rust dependencies are properly specified
- Verify the build script is executable (`chmod +x build-github-pages.sh`)

### Domain Not Working
- DNS changes can take up to 48 hours to propagate
- Use `dig bezy.org` to check DNS resolution
- Verify CNAME file is correctly generated in the dist folder

### WASM Not Loading
- Check browser console for errors
- Ensure CORS headers are properly set (GitHub Pages handles this automatically)
- Verify the WASM file size isn't too large (GitHub has limits)

### Assets Missing
- Ensure the `assets` directory exists in your repository
- Check that the build script successfully copies assets to `dist/assets/`

## Customization

### Changing the Domain
To use a different domain, edit the CNAME generation in `build-github-pages.sh`:
```bash
echo "yourdomain.com" > dist/CNAME
```

### Modifying the Loading Screen
Edit the HTML template in `build-github-pages.sh` to customize:
- Colors and styling
- Loading messages
- Company branding
- Error messages

### Build Optimization
For smaller file sizes, consider:
- Building with `--release` flag (change in build script)
- Enabling wasm-opt optimization
- Compressing assets

## Performance Considerations

- **First Load**: WASM apps have a longer initial load time
- **Caching**: GitHub Pages automatically caches static files
- **Compression**: Consider gzip compression for large WASM files
- **Progressive Loading**: The loading screen improves perceived performance

## Security Notes

- GitHub Pages serves over HTTPS automatically
- Custom domains get automatic SSL certificates
- No server-side processing, so no backend security concerns
- All code is public (it's a public repository)

## Maintenance

### Updating the Site
Simply push changes to the main branch:
```bash
git add .
git commit -m "Update font editor features"
git push origin main
```

### Monitoring
- GitHub provides basic analytics in repository insights
- Consider adding Google Analytics for detailed metrics
- Monitor GitHub Actions for build failures

## Advanced Configuration

### Multiple Environments
Set up staging deployments using branches:
```yaml
# In .github/workflows/deploy.yml
on:
  push:
    branches: [ "main", "staging" ]
```

### Custom Build Flags
Modify the build script for different optimization levels:
```bash
# For smaller file size (slower compile)
cargo build --target wasm32-unknown-unknown --release

# For development builds
cargo build --target wasm32-unknown-unknown
```

## Cost

GitHub Pages is free for public repositories, including:
- Unlimited bandwidth
- Custom domain support
- SSL certificates
- CDN distribution

## Support

For deployment issues:
1. Check GitHub Actions logs first
2. Review GitHub Pages documentation
3. Test the build script locally
4. Check DNS configuration with your domain provider

The deployment setup is designed to be robust and handle most common scenarios automatically. Once configured, you simply push code changes and the site updates automatically! 