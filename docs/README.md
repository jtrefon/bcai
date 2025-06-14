# BCAI Project Website

This is the official website for the BCAI (Blockchain AI) project - a decentralized AI training platform built on blockchain technology.

## 🌐 Live Website

The website is automatically deployed to GitHub Pages at: [https://jtrefon.github.io/bcai/](https://jtrefon.github.io/bcai/)

## 📁 Structure

```
docs/
├── index.html          # Main HTML file
├── styles.css          # CSS styles and responsive design
├── script.js           # JavaScript functionality and animations
├── favicon.svg         # Website favicon
└── README.md          # This file
```

## ✨ Features

### Design & UX
- **Modern, Clean Design** - Professional gradient-based color scheme
- **Fully Responsive** - Works perfectly on desktop, tablet, and mobile
- **Smooth Animations** - Intersection Observer API for scroll-triggered animations
- **Interactive Elements** - Hover effects, click animations, and parallax scrolling

### Navigation
- **Fixed Navigation Bar** - Stays accessible while scrolling
- **Smooth Scrolling** - Animated navigation between sections
- **Mobile Menu** - Hamburger menu for mobile devices

### Content Sections
1. **Hero Section** - Compelling headline with animated blockchain visualization
2. **Problem/Solution** - Clear narrative about AI training challenges and BCAI's solution
3. **Technology Stack** - Overview of technical components and architecture
4. **System Architecture** - Visual representation of the platform layers
5. **Development Roadmap** - Timeline with current progress and future milestones
6. **Contribute Section** - Clear calls-to-action for developers, researchers, and node operators
7. **Community Links** - Easy access to GitHub, discussions, and documentation

### Interactive Features
- **Animated Counters** - Statistics that count up when scrolled into view
- **Copy-to-Clipboard Code** - Click any code snippet to copy it
- **Dynamic Hover Effects** - Enhanced interactivity on cards and buttons
- **Progress Bar** - Shows reading progress as you scroll
- **Easter Egg** - Hidden Konami code activation (↑↑↓↓←→←→BA)

### Technical Features
- **GitHub API Integration** - Live repository statistics
- **Optimized Performance** - Minimal dependencies, efficient animations
- **SEO Optimized** - Proper meta tags, Open Graph, and Twitter Cards
- **Accessibility** - Semantic HTML and keyboard navigation support

## 🚀 Deployment

The website is automatically deployed via GitHub Actions when changes are pushed to the `main` branch in the `docs/` directory.

### GitHub Pages Setup
1. Go to repository Settings → Pages
2. Set Source to "GitHub Actions"
3. The workflow in `.github/workflows/pages.yml` handles the rest

### Manual Deployment
If you need to deploy manually:
1. Ensure all files are in the `docs/` directory
2. Push changes to the `main` branch
3. The GitHub Actions workflow will automatically deploy

## 🛠️ Development

### Local Development
To work on the website locally:

1. Clone the repository:
   ```bash
   git clone https://github.com/jtrefon/bcai.git
   cd bcai/docs
   ```

2. Serve the files using a local server:
   ```bash
   # Using Python
   python -m http.server 8000
   
   # Using Node.js
   npx serve .
   
   # Using PHP
   php -S localhost:8000
   ```

3. Open `http://localhost:8000` in your browser

### Making Changes

#### Content Updates
- **Text Content**: Edit `index.html` directly
- **Styling**: Modify `styles.css` for visual changes
- **Functionality**: Update `script.js` for interactive features

#### Adding New Sections
1. Add HTML structure in `index.html`
2. Add corresponding styles in `styles.css`
3. Add any interactive behavior in `script.js`
4. Update navigation menu if needed

#### Responsive Design
The website uses CSS Grid and Flexbox for responsive layouts. Test changes on:
- Desktop (1200px+)
- Tablet (768px - 1199px)
- Mobile (< 768px)

## 📱 Mobile Optimization

The website is fully optimized for mobile devices with:
- Touch-friendly navigation
- Responsive typography
- Optimized images and animations
- Mobile-first CSS approach

## 🎨 Brand Colors

```css
Primary Gradient: linear-gradient(135deg, #667eea 0%, #764ba2 100%)
Primary Blue: #667eea
Primary Purple: #764ba2
Background: #ffffff
Text: #333333
Light Gray: #f8f9fa
```

## 📊 Performance

The website is optimized for performance:
- **Lighthouse Score**: 95+ for Performance, Accessibility, Best Practices, SEO
- **Load Time**: < 2 seconds on 3G
- **Bundle Size**: < 100KB total (HTML + CSS + JS)

## 🔧 Maintenance

### Regular Updates
- Update roadmap milestones and progress
- Refresh GitHub statistics (automated via API)
- Update contributor information
- Add new community links

### Content Guidelines
- Keep technical explanations accessible
- Use clear calls-to-action
- Maintain consistent tone and style
- Ensure all links are working

## 🐛 Issues

If you find any issues with the website:
1. Check the [GitHub Issues](https://github.com/jtrefon/bcai/issues)
2. Create a new issue with the "website" label
3. Include browser version and device information

## 🤝 Contributing

To contribute to the website:
1. Fork the repository
2. Make your changes in the `docs/` directory
3. Test locally
4. Submit a pull request

## 📄 License

This website is part of the BCAI project and is licensed under the MIT License. 