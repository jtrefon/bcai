// BCAI Website JavaScript
document.addEventListener('DOMContentLoaded', function() {
    // Mobile Navigation Toggle
    const hamburger = document.querySelector('.hamburger');
    const navMenu = document.querySelector('.nav-menu');
    
    if (hamburger && navMenu) {
        hamburger.addEventListener('click', function() {
            hamburger.classList.toggle('active');
            navMenu.classList.toggle('active');
        });
        
        // Close mobile menu when clicking on links
        document.querySelectorAll('.nav-menu a').forEach(link => {
            link.addEventListener('click', function() {
                hamburger.classList.remove('active');
                navMenu.classList.remove('active');
            });
        });
    }
    
    // Smooth Scrolling for Navigation Links
    document.querySelectorAll('a[href^="#"]').forEach(anchor => {
        anchor.addEventListener('click', function (e) {
            e.preventDefault();
            const target = document.querySelector(this.getAttribute('href'));
            if (target) {
                const headerOffset = 80;
                const elementPosition = target.getBoundingClientRect().top;
                const offsetPosition = elementPosition + window.pageYOffset - headerOffset;
                
                window.scrollTo({
                    top: offsetPosition,
                    behavior: 'smooth'
                });
            }
        });
    });
    
    // Navbar Background on Scroll
    window.addEventListener('scroll', function() {
        const navbar = document.querySelector('.navbar');
        if (window.scrollY > 50) {
            navbar.style.background = 'rgba(255, 255, 255, 0.98)';
            navbar.style.boxShadow = '0 2px 20px rgba(0, 0, 0, 0.1)';
        } else {
            navbar.style.background = 'rgba(255, 255, 255, 0.95)';
            navbar.style.boxShadow = 'none';
        }
    });
    
    // Intersection Observer for Animations
    const observerOptions = {
        threshold: 0.1,
        rootMargin: '0px 0px -50px 0px'
    };
    
    const observer = new IntersectionObserver(function(entries) {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.style.opacity = '1';
                entry.target.style.transform = 'translateY(0)';
            }
        });
    }, observerOptions);
    
    // Observe elements for animation
    const animateElements = document.querySelectorAll('.problem-card, .tech-card, .contribute-card, .timeline-item');
    animateElements.forEach(el => {
        el.style.opacity = '0';
        el.style.transform = 'translateY(20px)';
        el.style.transition = 'opacity 0.6s ease, transform 0.6s ease';
        observer.observe(el);
    });
    
    // Counter Animation for Stats
    function animateCounter(element, target) {
        const duration = 2000;
        const increment = target / (duration / 16);
        let current = 0;
        
        const timer = setInterval(() => {
            current += increment;
            if (current >= target) {
                current = target;
                clearInterval(timer);
            }
            element.textContent = Math.floor(current);
        }, 16);
    }
    
    // Observe hero stats for counter animation
    const statsObserver = new IntersectionObserver(function(entries) {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                const statNumbers = entry.target.querySelectorAll('.stat-number');
                statNumbers.forEach(stat => {
                    const text = stat.textContent;
                    const number = parseInt(text.replace(/[^\d]/g, ''));
                    if (number) {
                        stat.textContent = '0';
                        animateCounter(stat, number);
                    }
                });
                statsObserver.unobserve(entry.target);
            }
        });
    }, { threshold: 0.5 });
    
    const heroStats = document.querySelector('.hero-stats');
    if (heroStats) {
        statsObserver.observe(heroStats);
    }
    
    // Copy to Clipboard for Code Snippets
    document.querySelectorAll('code').forEach(codeBlock => {
        codeBlock.style.cursor = 'pointer';
        codeBlock.title = 'Click to copy';
        
        codeBlock.addEventListener('click', function() {
            const text = this.textContent;
            navigator.clipboard.writeText(text).then(() => {
                // Show copied feedback
                const originalText = this.textContent;
                this.textContent = 'Copied!';
                this.style.background = '#4CAF50';
                this.style.color = 'white';
                
                setTimeout(() => {
                    this.textContent = originalText;
                    this.style.background = '#e9ecef';
                    this.style.color = '#333';
                }, 1000);
            });
        });
    });
    
    // Blockchain Animation Enhancement
    function enhanceBlockchainAnimation() {
        const blocks = document.querySelectorAll('.block');
        blocks.forEach((block, index) => {
            block.addEventListener('mouseenter', function() {
                this.style.transform = 'scale(1.2) rotate(5deg)';
                this.style.boxShadow = '0 10px 20px rgba(102, 126, 234, 0.4)';
            });
            
            block.addEventListener('mouseleave', function() {
                this.style.transform = 'scale(1)';
                this.style.boxShadow = 'none';
            });
        });
    }
    
    enhanceBlockchainAnimation();
    
    // Parallax Effect for Hero Section
    window.addEventListener('scroll', function() {
        const scrolled = window.pageYOffset;
        const parallax = document.querySelector('.hero-visual');
        if (parallax) {
            const speed = scrolled * 0.5;
            parallax.style.transform = `translateY(${speed}px)`;
        }
    });
    
    // Dynamic Background for Technology Cards
    document.querySelectorAll('.tech-card').forEach(card => {
        card.addEventListener('mouseenter', function() {
            this.style.background = 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)';
            this.style.color = 'white';
            this.style.transform = 'translateY(-10px) scale(1.02)';
        });
        
        card.addEventListener('mouseleave', function() {
            this.style.background = 'white';
            this.style.color = '#333';
            this.style.transform = 'translateY(0) scale(1)';
        });
    });
    
    // Roadmap Timeline Animation
    const timelineItems = document.querySelectorAll('.timeline-item');
    const timelineObserver = new IntersectionObserver(function(entries) {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.classList.add('animate-in');
                const content = entry.target.querySelector('.timeline-content');
                if (content) {
                    content.style.transform = 'translateX(0)';
                    content.style.opacity = '1';
                }
            }
        });
    }, { threshold: 0.3 });
    
    timelineItems.forEach(item => {
        const content = item.querySelector('.timeline-content');
        if (content) {
            content.style.transform = 'translateX(-20px)';
            content.style.opacity = '0';
            content.style.transition = 'transform 0.6s ease, opacity 0.6s ease';
        }
        timelineObserver.observe(item);
    });
    
    // Progress Bar for Page Scroll
    function updateProgressBar() {
        const scrolled = (window.scrollY / (document.documentElement.scrollHeight - window.innerHeight)) * 100;
        let progressBar = document.querySelector('.scroll-progress');
        
        if (!progressBar) {
            progressBar = document.createElement('div');
            progressBar.className = 'scroll-progress';
            progressBar.style.cssText = `
                position: fixed;
                top: 0;
                left: 0;
                width: ${scrolled}%;
                height: 3px;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                z-index: 9999;
                transition: width 0.3s ease;
            `;
            document.body.appendChild(progressBar);
        } else {
            progressBar.style.width = scrolled + '%';
        }
    }
    
    window.addEventListener('scroll', updateProgressBar);
    
    // Easter Egg: Konami Code
    let konamiCode = [];
    const konamiSequence = ['ArrowUp', 'ArrowUp', 'ArrowDown', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'ArrowLeft', 'ArrowRight', 'KeyB', 'KeyA'];
    
    document.addEventListener('keydown', function(e) {
        konamiCode.push(e.code);
        if (konamiCode.length > konamiSequence.length) {
            konamiCode.shift();
        }
        
        if (konamiCode.join(',') === konamiSequence.join(',')) {
            // Activate easter egg
            document.body.style.background = 'linear-gradient(45deg, #ff6b6b, #4ecdc4, #45b7d1, #f9ca24)';
            document.body.style.backgroundSize = '400% 400%';
            document.body.style.animation = 'rainbow 3s ease infinite';
            
            // Add rainbow animation
            const style = document.createElement('style');
            style.textContent = `
                @keyframes rainbow {
                    0% { background-position: 0% 50%; }
                    50% { background-position: 100% 50%; }
                    100% { background-position: 0% 50%; }
                }
            `;
            document.head.appendChild(style);
            
            // Show celebration message
            const celebration = document.createElement('div');
            celebration.textContent = '🎉 BCAI Easter Egg Activated! 🚀';
            celebration.style.cssText = `
                position: fixed;
                top: 50%;
                left: 50%;
                transform: translate(-50%, -50%);
                background: rgba(0, 0, 0, 0.9);
                color: white;
                padding: 20px 40px;
                border-radius: 10px;
                font-size: 24px;
                z-index: 10000;
                animation: fadeInOut 3s ease forwards;
            `;
            
            const fadeStyle = document.createElement('style');
            fadeStyle.textContent = `
                @keyframes fadeInOut {
                    0%, 100% { opacity: 0; transform: translate(-50%, -50%) scale(0.5); }
                    50% { opacity: 1; transform: translate(-50%, -50%) scale(1); }
                }
            `;
            document.head.appendChild(fadeStyle);
            document.body.appendChild(celebration);
            
            setTimeout(() => {
                celebration.remove();
                document.body.style.background = '#ffffff';
                document.body.style.animation = 'none';
            }, 3000);
            
            konamiCode = [];
        }
    });
    
    // Form Validation (if contact forms are added)
    function validateEmail(email) {
        const re = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
        return re.test(email);
    }
    
    // GitHub API Integration (for live stats)
    async function fetchGitHubStats() {
        try {
            const response = await fetch('https://api.github.com/repos/jtrefon/bcai');
            const data = await response.json();
            
            // Update stats if elements exist
            const starsElement = document.querySelector('.github-stars');
            const forksElement = document.querySelector('.github-forks');
            
            if (starsElement) starsElement.textContent = data.stargazers_count || '0';
            if (forksElement) forksElement.textContent = data.forks_count || '0';
        } catch (error) {
            console.log('GitHub API fetch failed:', error);
        }
    }
    
    // Call GitHub stats on load
    fetchGitHubStats();
    
    console.log('🚀 BCAI Website Loaded! Welcome to the future of decentralized AI training.');
}); 