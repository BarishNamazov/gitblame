/* ============================================================================
   git-blame-2.0 — Landing Page JavaScript
   Sophisticated AI™ approved client-side interactions.
   ============================================================================ */

(function () {
  'use strict';

  // -------------------------------------------------------------------------
  // Matrix Rain Background
  // -------------------------------------------------------------------------
  function initMatrixRain() {
    const canvas = document.getElementById('matrix-rain');
    if (!canvas) return;
    const ctx = canvas.getContext('2d');

    function resize() {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
    }
    resize();
    window.addEventListener('resize', resize);

    const chars = 'gitblame01forgivetherapygudアイウエオカキクケコサシスセソ'.split('');
    const fontSize = 14;
    let columns = Math.floor(canvas.width / fontSize);
    let drops = new Array(columns).fill(1);

    window.addEventListener('resize', () => {
      columns = Math.floor(canvas.width / fontSize);
      drops = new Array(columns).fill(1);
    });

    function draw() {
      ctx.fillStyle = 'rgba(10, 14, 23, 0.05)';
      ctx.fillRect(0, 0, canvas.width, canvas.height);
      ctx.fillStyle = '#00ff8740';
      ctx.font = fontSize + 'px Fira Code, monospace';

      for (let i = 0; i < drops.length; i++) {
        const text = chars[Math.floor(Math.random() * chars.length)];
        ctx.fillText(text, i * fontSize, drops[i] * fontSize);

        if (drops[i] * fontSize > canvas.height && Math.random() > 0.975) {
          drops[i] = 0;
        }
        drops[i]++;
      }
    }

    setInterval(draw, 50);
  }

  // -------------------------------------------------------------------------
  // Terminal Typing Animation
  // -------------------------------------------------------------------------
  function initTerminalAnimation() {
    const output = document.getElementById('terminal-output');
    if (!output) return;

    const lines = [
      { text: '$ git blame src/auth.py -L 47', class: 'term-prompt', delay: 50 },
      { text: '', class: '', delay: 400 },
      { text: '🔍 Analyzing line 47... SQL string concatenation in auth handler', class: 'term-info', delay: 30 },
      { text: '🧠 Sophisticated AI™ composing email...', class: 'term-warn', delay: 40 },
      { text: '   tone: passive-aggressive (per .gitblame config)', class: 'term-dim', delay: 20 },
      { text: '   severity: scorched-earth (raw SQL in auth = absolutely not)', class: 'term-dim', delay: 20 },
      { text: '', class: '', delay: 200 },
      { text: '📧 Email sent to jdeveloper@org.com', class: 'term-success', delay: 30 },
      { text: '   CC: engineering-all@org.com', class: 'term-dim', delay: 20 },
      { text: '   Subject: Re: Line 47 of src/auth.py', class: 'term-dim', delay: 20 },
      { text: '', class: '', delay: 300 },
      { text: '   "We hope this email finds you well, which is more', class: 'term-dim', delay: 25 },
      { text: '    than we can say for the code on line 47."', class: 'term-dim', delay: 25 },
      { text: '', class: '', delay: 300 },
      { text: '   P.S. The commit message was "quick fix lol."', class: 'term-error', delay: 30 },
      { text: '   It was neither quick, nor a fix, nor lol.', class: 'term-error', delay: 30 },
      { text: '', class: '', delay: 500 },
      { text: '✅ Blame delivered. Justice served in 2.3s.', class: 'term-success', delay: 30 },
      { text: '⚠️  HR has been CC\'d. (Just kidding. Or are we?)', class: 'term-warn', delay: 40 },
    ];

    let lineIndex = 0;
    let charIndex = 0;
    let currentElement = null;

    function typeNextChar() {
      if (lineIndex >= lines.length) {
        // Restart after a pause
        setTimeout(() => {
          output.innerHTML = '';
          lineIndex = 0;
          charIndex = 0;
          currentElement = null;
          typeNextChar();
        }, 5000);
        return;
      }

      const line = lines[lineIndex];

      if (charIndex === 0) {
        currentElement = document.createElement('div');
        if (line.class) currentElement.className = line.class;
        output.appendChild(currentElement);
      }

      if (charIndex < line.text.length) {
        currentElement.textContent += line.text[charIndex];
        charIndex++;
        // Scroll terminal to bottom
        output.parentElement.scrollTop = output.parentElement.scrollHeight;
        setTimeout(typeNextChar, line.delay || 30);
      } else {
        charIndex = 0;
        lineIndex++;
        const nextDelay = line.delay ? Math.min(line.delay * 3, 600) : 200;
        setTimeout(typeNextChar, nextDelay);
      }
    }

    // Start after page loads with a brief delay
    setTimeout(typeNextChar, 1200);
  }

  // -------------------------------------------------------------------------
  // Scroll-Triggered Animations (Intersection Observer)
  // -------------------------------------------------------------------------
  function initScrollAnimations() {
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            entry.target.classList.add('visible');
          }
        });
      },
      { threshold: 0.1, rootMargin: '0px 0px -50px 0px' }
    );

    document.querySelectorAll('.animate-in').forEach((el) => {
      observer.observe(el);
    });
  }

  // -------------------------------------------------------------------------
  // Counter Animation for Metrics
  // -------------------------------------------------------------------------
  function initCounterAnimations() {
    const counters = document.querySelectorAll('.metric-number[data-target]');

    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting && !entry.target.dataset.counted) {
            entry.target.dataset.counted = 'true';
            animateCounter(entry.target);
          }
        });
      },
      { threshold: 0.5 }
    );

    counters.forEach((counter) => observer.observe(counter));

    function animateCounter(el) {
      const target = parseInt(el.dataset.target, 10);
      const duration = 2000;
      const startTime = performance.now();

      function easeOutExpo(t) {
        return t === 1 ? 1 : 1 - Math.pow(2, -10 * t);
      }

      function update(currentTime) {
        const elapsed = currentTime - startTime;
        const progress = Math.min(elapsed / duration, 1);
        const easedProgress = easeOutExpo(progress);
        const current = Math.floor(easedProgress * target);

        el.textContent = current.toLocaleString();

        if (progress < 1) {
          requestAnimationFrame(update);
        } else {
          el.textContent = target.toLocaleString();
        }
      }

      requestAnimationFrame(update);
    }
  }

  // -------------------------------------------------------------------------
  // Copy to Clipboard
  // -------------------------------------------------------------------------
  function initCopyButtons() {
    document.querySelectorAll('.copy-btn').forEach((btn) => {
      btn.addEventListener('click', async () => {
        const text = btn.dataset.copy;
        try {
          await navigator.clipboard.writeText(text);
        } catch {
          // Fallback for older browsers
          const textarea = document.createElement('textarea');
          textarea.value = text;
          textarea.style.position = 'fixed';
          textarea.style.opacity = '0';
          document.body.appendChild(textarea);
          textarea.select();
          document.execCommand('copy');
          document.body.removeChild(textarea);
        }
        btn.classList.add('copied');
        setTimeout(() => btn.classList.remove('copied'), 2000);
      });
    });
  }

  // -------------------------------------------------------------------------
  // Smooth Scroll for Nav Links
  // -------------------------------------------------------------------------
  function initSmoothScroll() {
    document.querySelectorAll('a[href^="#"]').forEach((link) => {
      link.addEventListener('click', (e) => {
        const target = document.querySelector(link.getAttribute('href'));
        if (target) {
          e.preventDefault();
          target.scrollIntoView({ behavior: 'smooth', block: 'start' });
        }
      });
    });
  }

  // -------------------------------------------------------------------------
  // Konami Code Easter Egg: ↑↑↓↓←→←→BA
  // -------------------------------------------------------------------------
  function initKonamiCode() {
    const code = [
      'ArrowUp', 'ArrowUp',
      'ArrowDown', 'ArrowDown',
      'ArrowLeft', 'ArrowRight',
      'ArrowLeft', 'ArrowRight',
      'KeyB', 'KeyA',
    ];
    let position = 0;

    const overlay = document.getElementById('konami-overlay');
    const closeBtn = document.getElementById('konami-close');
    const easterEggText = document.getElementById('easter-egg-text');

    document.addEventListener('keydown', (e) => {
      if (e.code === code[position]) {
        position++;
        if (position === code.length) {
          activateEasterEgg();
          position = 0;
        }
      } else {
        position = 0;
      }
    });

    function activateEasterEgg() {
      if (overlay) {
        overlay.classList.add('active');
        overlay.setAttribute('aria-hidden', 'false');
      }
      if (easterEggText) {
        easterEggText.classList.add('revealed');
      }
    }

    if (closeBtn) {
      closeBtn.addEventListener('click', () => {
        overlay.classList.remove('active');
        overlay.setAttribute('aria-hidden', 'true');
      });
    }

    // Close on Escape
    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape' && overlay && overlay.classList.contains('active')) {
        overlay.classList.remove('active');
        overlay.setAttribute('aria-hidden', 'true');
      }
    });

    // Close on overlay background click
    if (overlay) {
      overlay.addEventListener('click', (e) => {
        if (e.target === overlay) {
          overlay.classList.remove('active');
          overlay.setAttribute('aria-hidden', 'true');
        }
      });
    }
  }

  // -------------------------------------------------------------------------
  // Fake Notification Toast
  // -------------------------------------------------------------------------
  function initFakeNotification() {
    const toast = document.getElementById('toast');
    if (!toast) return;

    // Show after some seconds of scrolling
    let shown = false;
    function maybeShowToast() {
      if (shown) return;
      if (window.scrollY > window.innerHeight * 0.3) {
        shown = true;
        window.removeEventListener('scroll', maybeShowToast);
        toast.classList.add('show');
        setTimeout(() => {
          toast.classList.remove('show');
        }, 4000);
      }
    }

    window.addEventListener('scroll', maybeShowToast, { passive: true });
  }

  // -------------------------------------------------------------------------
  // Nav Background on Scroll
  // -------------------------------------------------------------------------
  function initNavScroll() {
    const nav = document.querySelector('.nav');
    if (!nav) return;

    function updateNav() {
      if (window.scrollY > 50) {
        nav.style.borderBottomColor = 'rgba(255, 255, 255, 0.08)';
        nav.style.background = 'rgba(10, 14, 23, 0.95)';
      } else {
        nav.style.borderBottomColor = 'rgba(255, 255, 255, 0.05)';
        nav.style.background = 'rgba(10, 14, 23, 0.85)';
      }
    }

    window.addEventListener('scroll', updateNav, { passive: true });
  }

  // -------------------------------------------------------------------------
  // Initialize Everything
  // -------------------------------------------------------------------------
  function init() {
    initMatrixRain();
    initTerminalAnimation();
    initScrollAnimations();
    initCounterAnimations();
    initCopyButtons();
    initSmoothScroll();
    initKonamiCode();
    initFakeNotification();
    initNavScroll();
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
