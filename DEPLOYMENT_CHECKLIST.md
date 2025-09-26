# Website Deployment Checklist - omendb.io

## Prerequisites
- [x] Website built and tested (`apps/website/dist/` exists)
- [x] GitHub Pages workflow configured (`.github/workflows/deploy-website.yml`)
- [x] Domain omendb.io owned by user

## Step 1: GitHub Pages Configuration

1. **Go to GitHub repository settings**:
   ```
   https://github.com/omendb/omendb/settings/pages
   ```

2. **Configure source**:
   - Source: "GitHub Actions"
   - The workflow will auto-deploy on pushes to main

3. **Test deployment**:
   - Push any change to `apps/website/`
   - Check Actions tab for deployment status
   - Site will be live at `https://omendb.github.io/omendb`

## Step 2: Custom Domain Setup

1. **DNS Configuration** (in your domain registrar):
   ```
   Type: CNAME
   Name: @ (or omendb.io)
   Value: omendb.github.io
   ```

2. **GitHub Custom Domain**:
   - Go to repository settings → Pages
   - Custom domain: `omendb.io`
   - Check "Enforce HTTPS"

3. **Verification**:
   - DNS propagation takes 5-60 minutes
   - Test: `dig omendb.io` should show GitHub IPs

## Step 3: Launch Validation

**Test these URLs work**:
- `https://omendb.io` - Landing page
- `https://omendb.io/demo` - Interactive demo
- `https://omendb.io/blog/making-postgres-10x-faster` - Blog post
- `https://omendb.io/docs` - Documentation
- `https://omendb.io/early-access` - Signup form

**Performance Check**:
- Run Lighthouse audit (should be 90+ across all metrics)
- Test on mobile devices
- Verify early access form works

## Step 4: Launch Sequence

1. **Soft Launch**:
   - Share with close contacts first
   - Test all functionality
   - Fix any issues

2. **HackerNews Launch**:
   - Title: "We made PostgreSQL 10x faster with machine learning"
   - URL: `https://omendb.io/blog/making-postgres-10x-faster`
   - Post during peak hours (9-11am PST)

3. **Monitor**:
   - GitHub stars
   - Website analytics
   - Early access signups

## Troubleshooting

**Website not updating**:
- Check GitHub Actions for build errors
- Clear browser cache
- Wait 5-10 minutes for CDN propagation

**Custom domain not working**:
- Verify DNS with `dig omendb.io`
- Check GitHub Pages settings
- Try with/without www prefix

**Build failures**:
- Check Astro config and dependencies
- Ensure all imports are correct
- Test locally with `npm run build`

## Ready to Launch?

- [x] Website builds successfully
- [x] All pages render correctly
- [x] Interactive demo works
- [x] Documentation is complete
- [x] Early access form functions
- [x] GitHub Pages workflow configured

**Next**: Set up DNS, configure custom domain, launch!