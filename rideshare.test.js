const fs = require('fs');
const path = require('path');

function loadPage() {
  const html = fs.readFileSync(path.join(__dirname, 'rideshare.html'), 'utf8');
  document.documentElement.innerHTML = html;
}

beforeEach(() => {
  loadPage();
});

describe('Page metadata', () => {
  test('has correct title', () => {
    expect(document.title).toBe('Elite Tesla Transfers | Airport Transportation');
  });
});

describe('Navigation', () => {
  test('has a link to the services section', () => {
    const link = document.querySelector('nav a[href="#services"]');
    expect(link).not.toBeNull();
    expect(link.textContent.trim()).toBe('Services');
  });

  test('has a link to the contact section', () => {
    const link = document.querySelector('nav a[href="#contact"]');
    expect(link).not.toBeNull();
    expect(link.textContent.trim()).toBe('Contact');
  });
});

describe('Hero section', () => {
  test('hero heading mentions Tesla Model Y', () => {
    const h1 = document.querySelector('.hero h1');
    expect(h1).not.toBeNull();
    expect(h1.textContent).toMatch(/Tesla Model Y/i);
  });

  test('hero heading mentions airport transfers', () => {
    const h1 = document.querySelector('.hero h1');
    expect(h1.textContent).toMatch(/Airport Transfers/i);
  });

  test('hero CTA button links to contact section', () => {
    const btn = document.querySelector('.hero .btn');
    expect(btn).not.toBeNull();
    expect(btn.getAttribute('href')).toBe('#contact');
  });

  test('hero description mentions Tuscaloosa and Birmingham', () => {
    const p = document.querySelector('.hero p');
    expect(p.textContent).toMatch(/Tuscaloosa/i);
    expect(p.textContent).toMatch(/Birmingham/i);
  });
});

describe('Services section', () => {
  test('services section has the correct id', () => {
    expect(document.getElementById('services')).not.toBeNull();
  });

  test('displays fixed route service feature', () => {
    const features = Array.from(document.querySelectorAll('.feature-content h3'));
    const titles = features.map(h => h.textContent.trim());
    expect(titles).toContain('Fixed Route Service');
  });

  test('displays flat rate pricing feature', () => {
    const features = Array.from(document.querySelectorAll('.feature-content h3'));
    const titles = features.map(h => h.textContent.trim());
    expect(titles).toContain('Flat Rate Pricing');
  });

  test('displays easy scheduling feature', () => {
    const features = Array.from(document.querySelectorAll('.feature-content h3'));
    const titles = features.map(h => h.textContent.trim());
    expect(titles).toContain('Easy Scheduling');
  });

  test('flat rate pricing description mentions $80', () => {
    const features = Array.from(document.querySelectorAll('.feature'));
    const pricingFeature = features.find(f =>
      f.querySelector('h3')?.textContent.includes('Flat Rate Pricing')
    );
    expect(pricingFeature).not.toBeUndefined();
    expect(pricingFeature.textContent).toMatch(/\$80/);
  });

  test('displays $80 flat rate price element', () => {
    const priceEl = document.querySelector('#services .price');
    expect(priceEl).not.toBeNull();
    expect(priceEl.textContent).toMatch(/\$80/);
  });
});

describe('Contact section', () => {
  test('contact section has the correct id', () => {
    expect(document.getElementById('contact')).not.toBeNull();
  });

  test('phone link has correct tel href', () => {
    const phoneLink = document.querySelector('a[href="tel:2058264602"]');
    expect(phoneLink).not.toBeNull();
  });

  test('SMS link has correct sms href', () => {
    const smsLink = document.querySelector('a[href="sms:2058264602"]');
    expect(smsLink).not.toBeNull();
  });

  test('displays $80 pricing in contact section', () => {
    const contact = document.getElementById('contact');
    const priceEl = contact.querySelector('.price');
    expect(priceEl).not.toBeNull();
    expect(priceEl.textContent).toMatch(/\$80/);
  });

  test('contact card prompts call or text', () => {
    const cards = Array.from(document.querySelectorAll('.contact-card h3'));
    const headings = cards.map(h => h.textContent.trim());
    expect(headings).toContain('Call or Text Us');
  });

  test('contact card shows pricing information', () => {
    const cards = Array.from(document.querySelectorAll('.contact-card h3'));
    const headings = cards.map(h => h.textContent.trim());
    expect(headings).toContain('Pricing Information');
  });
});

describe('Footer', () => {
  test('footer exists', () => {
    expect(document.querySelector('footer')).not.toBeNull();
  });

  test('footer contains copyright notice', () => {
    const footer = document.querySelector('footer');
    expect(footer.textContent).toMatch(/Elite Tesla Transfers/i);
  });

  test('footer mentions Tesla Model Y', () => {
    const footer = document.querySelector('footer');
    expect(footer.textContent).toMatch(/Tesla Model Y/i);
  });
});
