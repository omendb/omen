# OmenDB Website

Professional website for OmenDB - the high-performance embedded vector database.

Live at: https://omendb.io

## Tech Stack

- **Framework**: SolidJS (lightweight, performant)
- **Build Tool**: Vite
- **Styling**: Tailwind CSS
- **Deployment**: GitHub Pages

## Development

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview
```

## Deployment

The site is deployed to omendb.io via Vercel/Netlify/your hosting provider.

For manual deployment:
```bash
# Build for production
npm run build

# Deploy the 'dist' directory to your hosting provider
```

For development preview:
```bash
npm run dev
```

## Structure

```
src/
├── components/     # Reusable components
├── pages/         # Page components
├── App.tsx        # Main app component
├── index.tsx      # Entry point
└── index.css      # Global styles
```

## Design Principles

1. **Minimal & Professional**: Clean design focused on content
2. **Performance First**: Built with SolidJS for optimal performance
3. **Developer Friendly**: Clear CTAs and code examples
4. **Scalable**: Can evolve into a full company website

## Future Enhancements

- Blog/News section
- Interactive demos
- API playground
- Multi-language support
- Server edition information